# Tire Swap Weather Station Finder

A Rust-based backend tool that helps determine optimal tire swap dates based on weather station data and climate metrics across Canada. The tool fetches real-time weather station data, stores it locally, and provides recommendations for when to switch between summer and winter tires.

## Features

- **Weather Station Data Collection**: Fetches active weather stations from Environment Canada's API
- **Climate Data Analysis**: Retrieves historical climate data including:
  - First snowfall dates
  - Last snowfall dates
  - Recommended summer tire switch dates
  - Recommended winter tire switch dates
- **Nearest Station Finder**: Uses KD-tree spatial indexing to quickly find the closest weather stations to any location
- **Local Database**: Stores all data in a SQLite database for offline access and faster queries
- **Smart Filtering**: Only includes stations that are currently active (reported data within the last week) and have at least 5 years of historical data

## Prerequisites

- **Rust**: Install from [rust-lang.org](https://rust-lang.org/tools/install/)
- **Cargo**: Comes with Rust installation

## Installation

1. Navigate to the backend directory:
   ```bash
   cd /path/to/tireswap/backend
   ```

2. Build the project:
   ```bash
   cargo build --release
   ```

## Usage

### First Time Setup: Populate the Database

Before using the tool, you need to populate the local database with weather station and climate data:

```bash
cargo run -- --update-db
```

This command will:
1. Fetch all active weather stations from Environment Canada
2. Store station information (location, name, coordinates) in the database
3. Retrieve climate data for each station (may take several minutes)
4. Display progress bars showing the collection status

**Note**: This process may take 5-15 minutes depending on your internet connection and the number of active stations.

### Find Nearest Weather Stations

After populating the database, run the tool without flags to find the nearest weather stations:

```bash
cargo run
```

By default, this will find the 5 nearest weather stations to the hardcoded coordinates (currently set to Windsor, ON: 42.3149, -83.0364) and display:
- Station name
- Distance in kilometers
- Coordinates
- Climate metrics (first/last snowfall, tire swap recommendations)

### Customize Location

To find weather stations near a different location, edit the coordinates in [src/main.rs](src/main.rs#L114-L115):

```rust
let home_lat = 42.3149;  // Your latitude
let home_lon = -83.0364; // Your longitude
```

Then rebuild and run:
```bash
cargo build
cargo run
```

## Command Line Options

```
Options:
  --update-db    Update the database with latest weather station and climate data
  -h, --help     Print help information
```

## Database

The tool creates a SQLite database file named `tireswap.db` in the backend directory. This file contains:
- **stations**: Weather station information (ID, name, coordinates, province)
- **climate_data**: Historical climate metrics for each station

To reset the database, simply delete the file and run `--update-db` again:

```bash
rm tireswap.db
cargo run -- --update-db
```

## Project Structure

```
backend/
├── Cargo.toml              # Rust dependencies and project configuration
├── src/
│   ├── main.rs            # Main entry point and CLI interface
│   ├── aggregator/        # Data fetching from Environment Canada API
│   │   ├── mod.rs
│   │   └── aggregator.rs
│   ├── db/                # Database operations and schema
│   │   ├── mod.rs
│   │   └── db.rs
│   └── nearest/           # KD-tree spatial search for finding nearest stations
│       ├── mod.rs
│       └── nearest.rs
└── tireswap.db           # SQLite database (created on first run)
```

## Dependencies

Key dependencies include:
- **reqwest**: HTTP client for API requests
- **tokio**: Async runtime
- **rusqlite**: SQLite database interface
- **kiddo**: KD-tree implementation for spatial searches
- **clap**: Command-line argument parsing
- **serde_json**: JSON parsing
- **indicatif**: Progress bars

See [Cargo.toml](Cargo.toml) for the complete list.

## Example Output

```
--- Finding Nearest Station ---
Finding 5 nearest stations to home (42.3149, -83.0364)...

1. WINDSOR A - 3.42 km away
   Location: (42.27, -82.96)
   Climate Metrics:
     First snowfall: November 15
     Last snowfall: March 28
     Switch to summer tires: April 15
     Switch to winter tires: November 1

2. CHATHAM KEIL DRIVE - 45.23 km away
   Location: (42.40, -82.18)
   Climate Metrics:
     First snowfall: November 20
     Last snowfall: March 25
     Switch to summer tires: April 10
     Switch to winter tires: November 5
...
```

## Performance

- Initial database population: ~5-15 minutes (one-time setup)
- Nearest station search: < 10ms (using KD-tree indexing)
- Concurrent API requests: Limited to 10 simultaneous connections to avoid overwhelming the server

## Data Source

All weather data is sourced from Environment Canada's climate data API. Stations are filtered to only include those that:
- Have reported data within the last 7 days (active stations)
- Have at least 5 years of historical data

## Troubleshooting

**Issue**: Database errors on startup  
**Solution**: Delete `tireswap.db` and run `cargo run -- --update-db` to recreate

**Issue**: No climate data for some stations  
**Solution**: Some stations may not have all climate metrics available - this is expected

**Issue**: API timeout errors  
**Solution**: Retry the `--update-db` command - the database will skip stations that already have data

## License

This project is for personal use.
