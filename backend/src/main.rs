use reqwest;

mod db;
use db::Database;

mod aggregator;
use aggregator::Aggregator;

#[tokio::main]
async fn get_data() -> Result<(), reqwest::Error> {
    let client = reqwest::Client::new();

    let query = [
        ("format", "csv"),
        ("stationID", "4607"),
        ("Year", "2023"),
        ("Month", "1"),
        ("Day", "1"),
        ("timeframe", "2"),
        ("submit", "Download Data"),
    ];

    let response = client
        .get("https://climate.weather.gc.ca/climate_data/bulk_data_e.html")
        .query(&query)
        .send()
        .await?
        .text()
        .await?;
    println!("Response: {}", response);
    Ok(())
}

#[tokio::main]
async fn main() {
    // Initialize database
    let db = Database::new("tireswap.db").unwrap();
    db.initialize_schema().unwrap();

    // Example aggregator usage
    let aggregator = Aggregator::new();

    match aggregator.get_sations().await {
        Ok(stations) => {
            println!("\nFetched stations from API:");
            for station in stations {
                match db.insert_station(station.id, &station.name, station.lon_x, station.lat_y) {
                    Ok(_) => continue,
                    Err(e) => eprintln!("Error inserting station ID {}: {}", station.id, e),
                }
            }
        }
        Err(e) => eprintln!("Error fetching stations: {}", e),
    }
}
