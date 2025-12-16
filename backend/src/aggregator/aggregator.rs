use crate::db::db;

use reqwest::Client;

pub struct Aggregator {
    // Aggregator implementation
    client: Client,
}

impl Aggregator {
    pub fn new() -> Self {
        // Initialization code
        let client = reqwest::Client::new();
        Aggregator { client }
    }

    /// Return list of Station structs
    pub async fn get_sations(&self) -> Result<Vec<db::Station>, reqwest::Error> {
        // Code to get stations

        let response = self
            .client
            .get("https://api.weather.gc.ca/collections/climate-stations/items?limit=99999")
            .send()
            .await?
            .text()
            .await?;

        let json: serde_json::Value = serde_json::from_str(&response).unwrap();

        let mut stations: Vec<db::Station> = Vec::new();

        for feature in json["features"].as_array().unwrap() {
            let properties = &feature["properties"];
            let station = db::Station {
                id: properties["STN_ID"].as_i64().unwrap(),
                name: properties["STATION_NAME"].as_str().unwrap().to_string(),
                lon_x: properties["LONGITUDE"].as_i64().unwrap() as f64 / 10000000.0,
                lat_y: properties["LATITUDE"].as_i64().unwrap() as f64 / 10000000.0,
            };
            stations.push(station);
        }

        Ok(stations)
    }
}
