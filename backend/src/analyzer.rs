use crate::db::Database;
use crate::nearest::NearestStationFinder;

#[derive(Debug, Clone)]
pub struct Recommendation {
    pub switch_to_summer: Option<String>,
    pub switch_to_winter: Option<String>,
    #[allow(dead_code)]
    pub latitude: f64,
    #[allow(dead_code)]
    pub longitude: f64,
    pub stations_analyzed: usize,
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

        for station in &nearest_stations {
            match self.db.get_data_by_station(station.id) {
                Ok(data_records) => {
                    if let Some(data) = data_records.first() {
                        if let Some(switch_summer) = &data.switch_to_summer {
                            summer_dates.push(switch_summer.clone());
                        }
                        if let Some(switch_winter) = &data.switch_to_winter {
                            winter_dates.push(switch_winter.clone());
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

        Ok(Recommendation {
            switch_to_summer,
            switch_to_winter,
            latitude,
            longitude,
            stations_analyzed: nearest_stations.len(),
        })
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
