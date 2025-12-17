use crate::db::Database;
use chrono::{Datelike, Duration, NaiveDate, NaiveDateTime, Utc};
use indicatif::{ProgressBar, ProgressStyle};
use reqwest::Client;

pub struct Aggregator<'a> {
    pub client: Client,
    pub db: &'a Database,
}

impl<'a> Aggregator<'a> {
    pub fn new(db: &'a Database) -> Self {
        let client = reqwest::Client::new();
        Aggregator { client, db }
    }

    /// Check if a station is still active (reported data within the last week)
    fn is_station_active(dly_last_date: Option<&str>) -> bool {
        let Some(date_str) = dly_last_date else {
            return false; // No date means not active
        };

        // Parse the date string (format: "YYYY-MM-DD HH:MM:SS")
        let parsed_date = NaiveDateTime::parse_from_str(date_str, "%Y-%m-%d %H:%M:%S")
            .ok()
            .or_else(|| {
                // Try parsing without time component
                chrono::NaiveDate::parse_from_str(date_str, "%Y-%m-%d")
                    .ok()
                    .map(|d| d.and_hms_opt(0, 0, 0).unwrap())
            });

        let Some(last_date) = parsed_date else {
            return false; // Couldn't parse date
        };

        let now = Utc::now().naive_utc();
        let one_week_ago = now - Duration::days(7);

        last_date >= one_week_ago
    }

    /// Check if a station has at least 5 years of data
    fn has_sufficient_data(dly_first_date: Option<&str>, dly_last_date: Option<&str>) -> bool {
        let Some(first_str) = dly_first_date else {
            return false;
        };
        let Some(last_str) = dly_last_date else {
            return false;
        };

        // Parse both dates
        let parse_date = |date_str: &str| -> Option<NaiveDateTime> {
            NaiveDateTime::parse_from_str(date_str, "%Y-%m-%d %H:%M:%S")
                .ok()
                .or_else(|| {
                    chrono::NaiveDate::parse_from_str(date_str, "%Y-%m-%d")
                        .ok()
                        .map(|d| d.and_hms_opt(0, 0, 0).unwrap())
                })
        };

        let Some(first_date) = parse_date(first_str) else {
            return false;
        };
        let Some(last_date) = parse_date(last_str) else {
            return false;
        };

        // Check if the difference is at least 5 years (1825 days to account for leap years)
        let five_years = Duration::days(1825);
        (last_date - first_date) >= five_years
    }

    /// Fetch stations from the API and insert them directly into the database
    /// Only includes stations that have reported data within the last week
    ///
    /// # Returns
    /// * `Result<usize, Box<dyn std::error::Error>>` - Number of stations inserted or error
    pub async fn fetch_and_store_stations(&self) -> Result<usize, Box<dyn std::error::Error>> {
        let response = self
            .client
            .get("https://api.weather.gc.ca/collections/climate-stations/items?limit=99999")
            .send()
            .await?
            .text()
            .await?;

        let json: serde_json::Value = serde_json::from_str(&response)?;

        let features = json["features"]
            .as_array()
            .ok_or("No features array in response")?;

        let pb = ProgressBar::new(features.len() as u64);
        pb.set_style(
            ProgressStyle::default_bar()
                .template("[{elapsed_precise}] {bar:40.cyan/blue} {pos}/{len} {msg}")
                .unwrap()
                .progress_chars("##-"),
        );
        pb.set_message("Processing stations...");

        let mut inserted_count = 0;
        let mut filtered_count = 0;
        let mut insufficient_data_count = 0;
        let mut total_count = 0;

        for feature in features {
            let properties = &feature["properties"];

            if let (Some(id), Some(name), Some(lon), Some(lat)) = (
                properties["STN_ID"].as_i64(),
                properties["STATION_NAME"].as_str(),
                properties["LONGITUDE"].as_i64(),
                properties["LATITUDE"].as_i64(),
            ) {
                total_count += 1;
                let lon_x = lon as f64 / 10000000.0;
                let lat_y = lat as f64 / 10000000.0;
                let dly_first_date = properties["DLY_FIRST_DATE"].as_str();
                let dly_last_date = properties["DLY_LAST_DATE"].as_str();

                // Filter out inactive stations
                if !Self::is_station_active(dly_last_date) {
                    filtered_count += 1;
                    pb.inc(1);
                    continue;
                }

                // Filter out stations with less than 5 years of data
                if !Self::has_sufficient_data(dly_first_date, dly_last_date) {
                    insufficient_data_count += 1;
                    pb.inc(1);
                    continue;
                }

                match self.db.insert_station(
                    id,
                    &name.to_string(),
                    lon_x,
                    lat_y,
                    dly_first_date,
                    dly_last_date,
                ) {
                    Ok(_) => inserted_count += 1,
                    Err(e) => eprintln!("Error inserting station ID {}: {}", id, e),
                }
            }
            pb.inc(1);
        }

        pb.finish_with_message("Station processing complete");
        println!("\nTotal stations processed: {}", total_count);
        println!("Active stations (inserted): {}", inserted_count);
        println!("Inactive stations (filtered out): {}", filtered_count);
        println!("Insufficient data (<5 years): {}", insufficient_data_count);

        Ok(inserted_count)
    }

    /// Fetch daily weather data for a station and calculate climate metrics
    ///
    /// # Arguments
    /// * `station_id` - The station ID
    /// * `station_name` - The station name
    ///
    /// # Returns
    /// * `Result<(), Box<dyn std::error::Error>>` - Ok if successful
    pub async fn fetch_and_store_climate_data(
        &self,
        station_id: i64,
        _station_name: &str,
    ) -> Result<(), Box<dyn std::error::Error>> {
        // Get last 5 years of data
        let end_date = Utc::now().naive_utc().date();
        let start_year = end_date.year() - 5;
        let end_year = end_date.year();

        // Build list of all (year, month) pairs to fetch
        let mut months_to_fetch = Vec::new();
        for year in start_year..=end_year {
            for month in 1..=12 {
                // Skip future months
                if year == end_year && month > end_date.month() as i32 {
                    break;
                }
                months_to_fetch.push((year, month));
            }
        }

        // Fetch all months concurrently
        let mut tasks = Vec::new();
        for (year, month) in months_to_fetch {
            let client = self.client.clone();
            let station_id_str = station_id.to_string();
            let year_str = year.to_string();
            let month_str = month.to_string();

            let task = tokio::spawn(async move {
                let query = [
                    ("format", "csv"),
                    ("stationID", station_id_str.as_str()),
                    ("Year", year_str.as_str()),
                    ("Month", month_str.as_str()),
                    ("Day", "1"),
                    ("timeframe", "2"), // Daily data
                    ("submit", "Download Data"),
                ];

                let response = match client
                    .get("https://climate.weather.gc.ca/climate_data/bulk_data_e.html")
                    .query(&query)
                    .send()
                    .await
                {
                    Ok(r) => match r.text().await {
                        Ok(text) => text,
                        Err(_) => return Vec::new(),
                    },
                    Err(_) => return Vec::new(),
                };

                let mut records = Vec::new();
                let mut rdr = csv::Reader::from_reader(response.as_bytes());
                for result in rdr.records() {
                    if let Ok(record) = result {
                        // Extract fields: Date is field 4, Mean Temp is field 13, Total Snow is field 21
                        if let Some(date_str) = record.get(4) {
                            if let Ok(date) = NaiveDate::parse_from_str(date_str, "%Y-%m-%d") {
                                let mean_temp = record.get(13).and_then(|s| {
                                    if s.is_empty() || s == "M" {
                                        None
                                    } else {
                                        s.parse::<f64>().ok()
                                    }
                                });
                                let total_snow = record.get(21).and_then(|s| {
                                    if s.is_empty() || s == "M" {
                                        None
                                    } else {
                                        s.parse::<f64>().ok()
                                    }
                                });

                                records.push(DailyRecord {
                                    date,
                                    mean_temp,
                                    total_snow,
                                });
                            }
                        }
                    }
                }
                records
            });
            tasks.push(task);
        }

        // Wait for all tasks to complete and collect results
        let mut all_records: Vec<DailyRecord> = Vec::new();
        for task in tasks {
            if let Ok(records) = task.await {
                all_records.extend(records);
            }
        }

        if all_records.is_empty() {
            return Ok(());
        }

        // Group data by year
        let mut yearly_data: std::collections::HashMap<i32, Vec<&DailyRecord>> =
            std::collections::HashMap::new();
        for record in &all_records {
            yearly_data
                .entry(record.date.year())
                .or_insert_with(Vec::new)
                .push(record);
        }

        // Calculate metrics for each year
        let mut switch_to_summer_days = Vec::new();
        let mut switch_to_winter_days = Vec::new();
        let mut first_snowfall_days = Vec::new();
        let mut last_snowfall_days = Vec::new();

        for (_year, records) in yearly_data.iter_mut() {
            records.sort_by_key(|r| r.date);

            // Find the day to switch from winter to summer tires:
            // The day after the last time the mean daily temperature was below 7°C (in spring)
            // We look for the last occurrence of temp < 7 before we get sustained warmth
            let mut last_below_7_in_spring = None;
            for (i, record) in records.iter().enumerate() {
                if let Some(temp) = record.mean_temp {
                    if temp < 7.0 {
                        // Check if this is in the first half of the year (spring transition)
                        if record.date.ordinal() <= 180 {
                            last_below_7_in_spring = Some(i);
                        }
                    }
                }
            }
            if let Some(idx) = last_below_7_in_spring {
                // The switch day is the day after the last below-7 day
                if idx + 1 < records.len() {
                    switch_to_summer_days.push(records[idx + 1].date.ordinal() as i32);
                }
            }

            // Find the day to switch from summer to winter tires:
            // The FIRST day in fall where temp > 7°C and the following day was < 7°C
            // Start looking from July onwards (day 182) to avoid catching spring transitions
            for i in 0..records.len().saturating_sub(1) {
                if let Some(day_of_year) = records.get(i).map(|r| r.date.ordinal()) {
                    // Only look at dates from July onwards (after day 182)
                    if day_of_year >= 182 {
                        if let (Some(temp_today), Some(temp_tomorrow)) =
                            (records[i].mean_temp, records[i + 1].mean_temp)
                        {
                            if temp_today > 7.0 && temp_tomorrow < 7.0 {
                                // This is the first fall transition from above to below 7°C
                                // The switch day is this day (the last day above 7°C before cold)
                                switch_to_winter_days.push(records[i].date.ordinal() as i32);
                                break;
                            }
                        }
                    }
                }
            }

            // Find first snowfall
            if let Some(record) = records
                .iter()
                .find(|r| r.total_snow.map_or(false, |s| s > 0.0))
            {
                first_snowfall_days.push(record.date.ordinal() as i32);
            }

            // Find last snowfall
            if let Some(record) = records
                .iter()
                .rev()
                .find(|r| r.total_snow.map_or(false, |s| s > 0.0))
            {
                last_snowfall_days.push(record.date.ordinal() as i32);
            }
        }

        // Calculate averages
        let avg_switch_to_summer = average_day_of_year(&switch_to_summer_days);
        let avg_switch_to_winter = average_day_of_year(&switch_to_winter_days);
        let avg_first_snow = average_day_of_year(&first_snowfall_days);
        let avg_last_snow = average_day_of_year(&last_snowfall_days);

        // Store in database (using current year as reference)
        let current_year = Utc::now().year() as i64;
        self.db.insert_data(
            station_id,
            current_year,
            avg_first_snow.as_deref(),
            avg_last_snow.as_deref(),
            avg_switch_to_summer.as_deref(),
            avg_switch_to_winter.as_deref(),
        )?;

        Ok(())
    }
}

/// Helper struct for daily weather records
struct DailyRecord {
    date: NaiveDate,
    mean_temp: Option<f64>,
    total_snow: Option<f64>,
}

/// Calculate average day of year and convert to date string
fn average_day_of_year(days: &[i32]) -> Option<String> {
    if days.is_empty() {
        return None;
    }

    let sum: i32 = days.iter().sum();
    let avg_day = sum / days.len() as i32;

    // Convert day of year to date (using a non-leap year for simplicity)
    if let Some(date) = NaiveDate::from_yo_opt(2023, avg_day as u32) {
        Some(date.format("%Y-%m-%d").to_string())
    } else {
        None
    }
}
