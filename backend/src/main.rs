use reqwest;

mod db;
use db::Database;

mod aggregator;
use aggregator::Aggregator;

mod nearest;
use nearest::NearestStationFinder;

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
    // Get bulk data example
    let _ = get_data().await.unwrap();

    // Initialize database
    let db = Database::new("tireswap.db").unwrap();
    db.initialize_schema().unwrap();

    // Fetch and store stations using aggregator
    let aggregator = Aggregator::new(&db);

    println!("\nFetching stations from API...");
    match aggregator.fetch_and_store_stations().await {
        Ok(count) => println!("Successfully inserted {} stations into database", count),
        Err(e) => eprintln!("Error fetching/storing stations: {}", e),
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
                if let (Some(first), Some(last)) = (&nearest.dly_first_date, &nearest.dly_last_date) {
                    println!("  Daily data available: {} to {}", first, last);
                }
            } else {
                println!("No stations found in database");
            }
            
            // Find 5 nearest stations
            println!("\nFinding 5 nearest stations...");
            let nearest_5 = finder.find_k_nearest(home_lat, home_lon, 5);
            for (i, station) in nearest_5.iter().enumerate() {
                let date_info = match (&station.dly_first_date, &station.dly_last_date) {
                    (Some(first), Some(last)) => format!(" (data: {} to {})", first, last),
                    _ => String::new(),
                };
                println!("{}. {} - {:.2} km away{}", i + 1, station.name, station.distance_km, date_info);
            }
        }
        Err(e) => eprintln!("Error creating nearest station finder: {}", e),
    }
}
