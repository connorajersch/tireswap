use clap::Parser;
use indicatif::{ProgressBar, ProgressStyle};
use std::sync::Arc;

mod db;
use db::Database;

mod aggregator;
use aggregator::Aggregator;

mod nearest;

mod analyzer;
use analyzer::Analyzer;

mod api;
use api::{AppState, create_router};

/// Tire Swap Weather Station Finder
#[derive(Parser, Debug)]
#[command(name = "backend")]
#[command(about = "Find nearest weather stations and climate data for tire swap recommendations", long_about = None)]
struct Args {
    /// Update the database with latest weather station and climate data
    #[arg(long)]
    update_db: bool,

    /// Latitude of the location to analyze
    #[arg(long)]
    latitude: Option<f64>,

    /// Longitude of the location to analyze
    #[arg(long)]
    longitude: Option<f64>,

    /// Number of nearest stations to consider for analysis
    #[arg(long, short = 'n', default_value = "5")]
    num_stations: usize,

    /// Run as API server
    #[arg(long)]
    serve: bool,

    /// Port to run the API server on
    #[arg(long, default_value = "3000")]
    port: u16,

    /// Database file path
    #[arg(long, default_value = "tireswap.db")]
    db_path: String,
}

#[tokio::main]
async fn main() {
    let args = Args::parse();

    // Initialize database
    let db = Database::new(&args.db_path).unwrap();
    db.initialize_schema().unwrap();

    // If serve mode is enabled, start the API server
    if args.serve {
        run_server(db, args.port).await;
        return;
    }

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

    // Analyze tire swap dates for a location (if coordinates provided)
    if let (Some(latitude), Some(longitude)) = (args.latitude, args.longitude) {
        println!("\n--- Tire Swap Analysis ---");
        match Analyzer::new(&db) {
            Ok(analyzer) => {
                println!(
                    "Analyzing tire swap dates for location ({}, {})...\n",
                    latitude, longitude
                );

                match analyzer.analyze(latitude, longitude, args.num_stations) {
                    Ok(recommendation) => {
                        println!(
                            "Based on {} nearest weather stations:",
                            recommendation.stations_analyzed
                        );
                        println!();

                        if let Some(summer) = recommendation.switch_to_summer {
                            println!("🌞 Switch to summer tires: {}", summer);
                        } else {
                            println!("🌞 Switch to summer tires: No data available");
                        }

                        if let Some(winter) = recommendation.switch_to_winter {
                            println!("❄️  Switch to winter tires: {}", winter);
                        } else {
                            println!("❄️  Switch to winter tires: No data available");
                        }
                        println!();
                    }
                    Err(e) => eprintln!("Error analyzing tire swap dates: {}", e),
                }
            }
            Err(e) => eprintln!("Error creating tire swap analyzer: {}", e),
        }
    } else if !args.update_db {
        eprintln!("\nError: Please provide --latitude and --longitude to analyze a location.");
        eprintln!("Or use --update-db to update the database.");
        eprintln!("Or use --serve to start the API server.\n");
        eprintln!("For help, run: cargo run -- --help");
    }
}

/// Run the API server
async fn run_server(db: Database, port: u16) {
    let db_arc = Arc::new(db);
    let state = AppState { db: db_arc };
    let app = create_router(state);

    let addr = format!("0.0.0.0:{}", port);
    let listener = tokio::net::TcpListener::bind(&addr)
        .await
        .expect("Failed to bind to address");

    println!("🚀 Tire Swap API server running on http://{}", addr);
    println!("   Health check: http://{}/health", addr);
    println!("   Optimal dates: http://{}/api/optimal-dates?latitude=<lat>&longitude=<lon>", addr);
    println!();

    axum::serve(listener, app)
        .await
        .expect("Server failed to start");
}
