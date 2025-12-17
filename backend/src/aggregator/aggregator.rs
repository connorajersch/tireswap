use crate::db::Database;
use reqwest::Client;
use chrono::{NaiveDateTime, Utc, Duration};

pub struct Aggregator<'a> {
    client: Client,
    db: &'a Database,
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

        let mut inserted_count = 0;
        let mut filtered_count = 0;
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
                    continue;
                }
                
                match self.db.insert_station(id, &name.to_string(), lon_x, lat_y, dly_first_date, dly_last_date) {
                    Ok(_) => inserted_count += 1,
                    Err(e) => eprintln!("Error inserting station ID {}: {}", id, e),
                }
            }
        }

        println!("Total stations processed: {}", total_count);
        println!("Active stations (inserted): {}", inserted_count);
        println!("Inactive stations (filtered out): {}", filtered_count);

        Ok(inserted_count)
    }
}
