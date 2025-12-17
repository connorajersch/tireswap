use reqwest;

mod db;
use db::Database;

mod aggregator;
use aggregator::Aggregator;

mod nearest;
use nearest::NearestStationFinder;

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
            println!("\nFetched stations from API...");
            for station in stations {
                match db.insert_station(station.id, &station.name, station.lon_x, station.lat_y) {
                    Ok(_) => continue,
                    Err(e) => eprintln!("Error inserting station ID {}: {}", station.id, e),
                }
            }
        }
        Err(e) => eprintln!("Error fetching stations: {}", e),
    }

    // Example: Find nearest station to Toronto coordinates
    println!("\n--- Finding Nearest Station ---");
    match NearestStationFinder::new(&db) {
        Ok(finder) => {
            // home: 42.2362669,-83.0227593,
            let home_lat = 42.2362669;
            let home_lon = -83.0227593;
            
            println!("Finding nearest station to home ({}, {})...", home_lat, home_lon);
            
            if let Some(nearest) = finder.find_nearest(home_lat, home_lon) {
                println!("Nearest station: {} (ID: {})", nearest.name, nearest.id);
                println!("  Location: ({}, {})", nearest.lat_y, nearest.lon_x);
                println!("  Distance: {:.2} km", nearest.distance_km);
            } else {
                println!("No stations found in database");
            }
            
            // Find 5 nearest stations
            println!("\nFinding 5 nearest stations...");
            let nearest_5 = finder.find_k_nearest(home_lat, home_lon, 5);
            for (i, station) in nearest_5.iter().enumerate() {
                println!("{}. {} - {:.2} km away", i + 1, station.name, station.distance_km);
            }
        }
        Err(e) => eprintln!("Error creating nearest station finder: {}", e),
    }
}
