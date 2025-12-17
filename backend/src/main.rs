use clap::Parser;
use indicatif::{ProgressBar, ProgressStyle};
use std::sync::Arc;

mod db;
use db::Database;

mod aggregator;
use aggregator::Aggregator;

mod nearest;
use nearest::NearestStationFinder;

/// Tire Swap Weather Station Finder
#[derive(Parser, Debug)]
#[command(name = "backend")]
#[command(about = "Find nearest weather stations and climate data for tire swap recommendations", long_about = None)]
struct Args {
    /// Update the database with latest weather station and climate data
    #[arg(long)]
    update_db: bool,
}

#[tokio::main]
async fn main() {
    let args = Args::parse();

    // Initialize database
    let db = Database::new("tireswap.db").unwrap();
    db.initialize_schema().unwrap();

    // Fetch and store stations using aggregator if --update-db flag is passed
    if args.update_db {
        let aggregator = Arc::new(Aggregator::new(&db));

        println!("\nFetching stations from API...");
        match aggregator.fetch_and_store_stations().await {
            Ok(count) => {
                println!("Successfully inserted {} stations into database", count);

                // Fetch climate data for all stations
                println!("\nFetching climate data for all stations...");
                match db.get_all_stations() {
                    Ok(stations) => {
                        let pb = ProgressBar::new(stations.len() as u64);
                        pb.set_style(
                            ProgressStyle::default_bar()
                                .template(
                                    "[{elapsed_precise}] {bar:40.cyan/blue} {pos}/{len} {msg}",
                                )
                                .unwrap()
                                .progress_chars("##-"),
                        );

                        // Process stations with controlled concurrency using buffered stream
                        let concurrent_limit = 10;
                        use futures::stream::{self, StreamExt};

                        let agg = Arc::clone(&aggregator);
                        let mut stream = stream::iter(stations)
                            .map(|station| {
                                let station_id = station.id;
                                let station_name = station.name.clone();
                                let agg = Arc::clone(&agg);
                                async move {
                                    let result = agg
                                        .fetch_and_store_climate_data(station_id, &station_name)
                                        .await;
                                    (result, station_name)
                                }
                            })
                            .buffer_unordered(concurrent_limit);

                        // Process results as they complete
                        while let Some((result, name)) = stream.next().await {
                            if let Err(e) = result {
                                pb.println(format!("  ✗ Error for {}: {}", name, e));
                            }
                            pb.inc(1);
                        }

                        pb.finish_with_message("Climate data collection complete!");
                        println!();
                    }
                    Err(e) => eprintln!("Error retrieving stations: {}", e),
                }
            }
            Err(e) => eprintln!("Error fetching/storing stations: {}", e),
        }
    }

    // Example: Find nearest station to Home coordinates
    println!("\n--- Finding Nearest Station ---");
    match NearestStationFinder::new(&db) {
        Ok(finder) => {
            // Home coordinates Windsor, ON
            let home_lat = 42.3149;
            let home_lon = -83.0364;

            // Home coordinates London, ON
            // let home_lat = 42.9849;
            // let home_lon = -81.2453;

            // Home coordinates Thunder Bay, ON
            // let home_lat = 48.3809;
            // let home_lon = -89.2477;

            // Home coordinates Winnipeg, MB
            // let home_lat = 49.8951;
            // let home_lon = -97.1384;

            // Home coordinates Calgary, AB
            // let home_lat = 51.0447;
            // let home_lon = -114.0719;

            println!(
                "Finding 5 nearest stations to home ({}, {})...\n",
                home_lat, home_lon
            );
            let nearest_5 = finder.find_k_nearest(home_lat, home_lon, 5);

            for (i, station) in nearest_5.iter().enumerate() {
                println!(
                    "{}. {} - {:.2} km away",
                    i + 1,
                    station.name,
                    station.distance_km
                );
                println!("   Location: ({}, {})", station.lat_y, station.lon_x);

                // Fetch climate data for this station
                match db.get_data_by_station(station.id) {
                    Ok(data_records) => {
                        if let Some(data) = data_records.first() {
                            println!("   Climate Metrics:");
                            if let Some(first_snow) = &data.first_sf {
                                println!("     First snowfall: {}", first_snow);
                            } else {
                                println!("     First snowfall: N/A");
                            }
                            if let Some(last_snow) = &data.last_sf {
                                println!("     Last snowfall: {}", last_snow);
                            } else {
                                println!("     Last snowfall: N/A");
                            }
                            if let Some(switch_summer) = &data.switch_to_summer {
                                println!("     Switch to summer tires: {}", switch_summer);
                            } else {
                                println!("     Switch to summer tires: N/A");
                            }
                            if let Some(switch_winter) = &data.switch_to_winter {
                                println!("     Switch to winter tires: {}", switch_winter);
                            } else {
                                println!("     Switch to winter tires: N/A");
                            }
                        } else {
                            println!("   Climate Metrics: No data available");
                        }
                    }
                    Err(e) => println!("   Climate Metrics: Error fetching data - {}", e),
                }
                println!();
            }
        }
        Err(e) => eprintln!("Error creating nearest station finder: {}", e),
    }
}
