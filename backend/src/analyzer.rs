use crate::db::Database;
use crate::nearest::{NearestStationFinder, StationWithDistance};

#[derive(Debug, Clone)]
pub struct Recommendation {
    pub switch_to_summer: Option<String>,
    pub switch_to_winter: Option<String>,
    #[allow(dead_code)]
    pub latitude: f64,
    #[allow(dead_code)]
    pub longitude: f64,
    pub stations_analyzed: usize,
    pub stations_requested: usize,
    pub stations: Vec<StationWithDistance>,
    pub summer_stations_with_data: usize,
    pub winter_stations_with_data: usize,
    pub data_years: DataYearsStats,
}

#[derive(Debug, Clone)]
pub struct DataYearsStats {
    pub min_span_years: Option<i64>,
    pub avg_span_years: Option<f64>,
    pub max_span_years: Option<i64>,
}

pub struct Analyzer<'a> {
    db: &'a Database,
    finder: NearestStationFinder,
}

impl<'a> Analyzer<'a> {
    pub fn new(db: &'a Database) -> Result<Self, Box<dyn std::error::Error>> {
        let finder = NearestStationFinder::new(db)?;
        Ok(Self { db, finder })
    }

    /// Analyze tire swap dates for a given location
    /// 
    /// # Arguments
    /// * `latitude` - Latitude of the location
    /// * `longitude` - Longitude of the location
    /// * `num_stations` - Number of nearest stations to consider (default: 5)
    pub fn analyze(
        &self,
        latitude: f64,
        longitude: f64,
        num_stations: usize,
    ) -> Result<Recommendation, Box<dyn std::error::Error>> {
        let nearest_stations = self
            .finder
            .find_k_nearest(latitude, longitude, num_stations);

        let mut summer_dates = Vec::new();
        let mut winter_dates = Vec::new();
        let mut summer_stations_with_data = 0;
        let mut winter_stations_with_data = 0;

        for station in &nearest_stations {
            match self.db.get_data_by_station(station.id) {
                Ok(data_records) => {
                    if let Some(data) = data_records.first() {
                        if let Some(switch_summer) = &data.switch_to_summer {
                            summer_dates.push(switch_summer.clone());
                            summer_stations_with_data += 1;
                        }
                        if let Some(switch_winter) = &data.switch_to_winter {
                            winter_dates.push(switch_winter.clone());
                            winter_stations_with_data += 1;
                        }
                    }
                }
                Err(_) => continue,
            }
        }

        let switch_to_summer = if !summer_dates.is_empty() {
            calculate_average_date(&summer_dates)
        } else {
            None
        };

        let switch_to_winter = if !winter_dates.is_empty() {
            calculate_average_date(&winter_dates)
        } else {
            None
        };

        let data_years = calculate_data_years_stats(&nearest_stations);

        Ok(Recommendation {
            switch_to_summer,
            switch_to_winter,
            latitude,
            longitude,
            stations_analyzed: nearest_stations.len(),
            stations_requested: num_stations,
            stations: nearest_stations,
            summer_stations_with_data,
            winter_stations_with_data,
            data_years,
        })
    }
}

fn parse_year_from_date(date_str: &str) -> Option<i64> {
    if let Some(year_part) = date_str.split('-').next() {
        if year_part.len() == 4 {
            return year_part.parse().ok();
        }
    }
    None
}

fn calculate_data_years_stats(stations: &[StationWithDistance]) -> DataYearsStats {
    let mut spans: Vec<i64> = Vec::new();

    for station in stations {
        let start_year = station
            .dly_first_date
            .as_deref()
            .and_then(parse_year_from_date);
        let end_year = station
            .dly_last_date
            .as_deref()
            .and_then(parse_year_from_date);

        if let (Some(start), Some(end)) = (start_year, end_year) {
            if end >= start {
                spans.push(end - start + 1);
            }
        }
    }

    if spans.is_empty() {
        return DataYearsStats {
            min_span_years: None,
            avg_span_years: None,
            max_span_years: None,
        };
    }

    let min_span_years = spans.iter().min().copied();
    let max_span_years = spans.iter().max().copied();
    let sum: i64 = spans.iter().sum();
    let avg_span_years = Some(sum as f64 / spans.len() as f64);

    DataYearsStats {
        min_span_years,
        avg_span_years,
        max_span_years,
    }
}

/// Parse a date string (e.g., "2023-04-15" or "April 15") and return day of year
fn parse_date_to_day_of_year(date_str: &str) -> Option<u32> {
    // Try parsing ISO format first (YYYY-MM-DD)
    if date_str.contains('-') {
        let parts: Vec<&str> = date_str.split('-').collect();
        if parts.len() == 3 {
            let month: u32 = parts[1].parse().ok()?;
            let day: u32 = parts[2].parse().ok()?;

            // Calculate day of year (assuming non-leap year for simplicity)
            let days_before_month = [0, 31, 59, 90, 120, 151, 181, 212, 243, 273, 304, 334];
            if month >= 1 && month <= 12 {
                return Some(days_before_month[(month - 1) as usize] + day);
            }
        }
    }

    // Try parsing "Month Day" format
    let parts: Vec<&str> = date_str.split_whitespace().collect();
    if parts.len() != 2 {
        return None;
    }

    let month = match parts[0] {
        "January" => 1,
        "February" => 2,
        "March" => 3,
        "April" => 4,
        "May" => 5,
        "June" => 6,
        "July" => 7,
        "August" => 8,
        "September" => 9,
        "October" => 10,
        "November" => 11,
        "December" => 12,
        _ => return None,
    };

    let day: u32 = parts[1].parse().ok()?;

    // Calculate day of year (assuming non-leap year for simplicity)
    let days_before_month = [0, 31, 59, 90, 120, 151, 181, 212, 243, 273, 304, 334];
    Some(days_before_month[(month - 1) as usize] + day)
}

/// Convert day of year back to "Month Day" format
fn day_of_year_to_date(day: u32) -> String {
    let days_in_months = [31, 28, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31];
    let month_names = [
        "January",
        "February",
        "March",
        "April",
        "May",
        "June",
        "July",
        "August",
        "September",
        "October",
        "November",
        "December",
    ];

    let mut remaining = day;
    for (i, &days) in days_in_months.iter().enumerate() {
        if remaining <= days {
            return format!("{} {}", month_names[i], remaining);
        }
        remaining -= days;
    }
    "Invalid date".to_string()
}

/// Calculate the average date from a list of date strings
fn calculate_average_date(dates: &[String]) -> Option<String> {
    let days: Vec<u32> = dates
        .iter()
        .filter_map(|d| parse_date_to_day_of_year(d))
        .collect();

    if days.is_empty() {
        return None;
    }

    let sum: u32 = days.iter().sum();
    let avg = sum / days.len() as u32;
    Some(day_of_year_to_date(avg))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::Database;

    #[test]
    fn test_data_years_stats() {
        let stations = vec![
            StationWithDistance {
                id: 1,
                name: "A".to_string(),
                lon_x: 0.0,
                lat_y: 0.0,
                distance_km: 1.0,
                dly_first_date: Some("2010-01-01".to_string()),
                dly_last_date: Some("2019-12-31".to_string()),
            },
            StationWithDistance {
                id: 2,
                name: "B".to_string(),
                lon_x: 1.0,
                lat_y: 1.0,
                distance_km: 2.0,
                dly_first_date: Some("2015-01-01".to_string()),
                dly_last_date: Some("2020-12-31".to_string()),
            },
        ];

        let stats = calculate_data_years_stats(&stations);
        assert_eq!(stats.min_span_years, Some(6));
        assert_eq!(stats.max_span_years, Some(10));
        assert!(stats.avg_span_years.unwrap() > 7.0);
    }

    #[test]
    fn test_coverage_counts() {
        let db = Database::new_in_memory().unwrap();
        db.initialize_schema().unwrap();

        db.insert_station(1, &"Station 1".to_string(), -79.4, 43.7, None, None)
            .unwrap();
        db.insert_station(2, &"Station 2".to_string(), -79.5, 43.8, None, None)
            .unwrap();

        db.insert_data(1, 2023, Some("2023-04-15"), None)
            .unwrap();
        db.insert_data(2, 2023, None, Some("2023-11-01"))
            .unwrap();

        let analyzer = Analyzer::new(&db).unwrap();
        let rec = analyzer.analyze(43.7, -79.4, 2).unwrap();

        assert_eq!(rec.stations_analyzed, 2);
        assert_eq!(rec.summer_stations_with_data, 1);
        assert_eq!(rec.winter_stations_with_data, 1);
    }

    #[test]
    fn test_analyze_uses_latest_year_data_per_station() {
        let db = Database::new_in_memory().unwrap();
        db.initialize_schema().unwrap();

        db.insert_station(1, &"Station 1".to_string(), -79.4, 43.7, None, None)
            .unwrap();

        db.insert_data(1, 2020, Some("2020-04-01"), Some("2020-10-01"))
            .unwrap();
        db.insert_data(1, 2024, Some("2024-05-15"), Some("2024-11-15"))
            .unwrap();

        let analyzer = Analyzer::new(&db).unwrap();
        let rec = analyzer.analyze(43.7, -79.4, 1).unwrap();

        assert_eq!(rec.switch_to_summer.as_deref(), Some("May 15"));
        assert_eq!(rec.switch_to_winter.as_deref(), Some("November 15"));
    }
}
