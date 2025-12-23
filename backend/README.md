# Tire Swap Weather Station Finder

A Rust-based backend tool and REST API that helps determine optimal tire swap dates based on weather station data and climate metrics across Canada. The tool fetches real-time weather station data, stores it locally, and analyzes climate patterns to provide personalized recommendations for when to switch between summer and winter tires based on your location.

## Features

- **REST API Server**: Run as a web service to provide tire swap recommendations via HTTP endpoints
- **Weather Station Data Collection**: Fetches active weather stations from Environment Canada's API
- **Climate Data Analysis**: Retrieves historical climate data including:
  - First snowfall dates
  - Last snowfall dates
  - Recommended summer tire switch dates
  - Recommended winter tire switch dates
- **Tire Swap Analyzer**: Analyzes the k-nearest weather stations to your location and calculates average optimal tire change dates
- **Nearest Station Finder**: Uses KD-tree spatial indexing to quickly find the closest weather stations to any location
- **Local Database**: Stores all data in a SQLite database for offline access and faster queries
- **Smart Filtering**: Only includes stations that are currently active (reported data within the last week) and have at least 5 years of historical data
- **Thread-Safe**: Uses Mutex-protected database connections for safe concurrent access

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

### Running as a REST API Server

Start the API server to provide tire swap recommendations via HTTP:

```bash
# Start the server on default port (3000)
cargo run -- --serve

# Start on a custom port
cargo run -- --serve --port 8080

# Use a custom database file
cargo run -- --serve --db-path /path/to/custom.db
```

Once running, the API will be available at `http://localhost:3000` (or your custom port).

**API Endpoints:**

- `GET /health` - Health check endpoint
- `GET /api/optimal-dates?latitude={lat}&longitude={lon}&num_stations={n}` - Get tire swap recommendations

**Example API Requests:**

```bash
# Health check
curl http://localhost:3000/health

# Get recommendations for Toronto
curl "http://localhost:3000/api/optimal-dates?latitude=43.7&longitude=-79.4"

# Get recommendations for Vancouver using 10 stations
curl "http://localhost:3000/api/optimal-dates?latitude=49.28&longitude=-123.12&num_stations=10"
```

**Test Script:**

A test script is provided to quickly test all API endpoints:

```bash
./test_api.sh
```

For complete API documentation, see [API.md](API.md).

### First Time Setup: Populate the Database

Before using the tool (CLI or API), you need to populate the local database with weather station and climate data:

```bash
cargo run -- --update-db
```

This command will:
1. Fetch all active weather stations from Environment Canada
2. Store station information (location, name, coordinates) in the database
3. Retrieve climate data for each station (may take several minutes)
4. Display progress bars showing the collection status

**Note**: This process may take 5-15 minutes depending on your internet connection and the number of active stations.

### CLI Mode: Get Tire Swap Recommendations

After populating the database, you can run the tool in CLI mode with your location coordinates to get tire swap recommendations:

```bash
# Analyze Windsor, ON
cargo run -- --latitude 42.3149 --longitude=-83.0364

# Analyze Calgary, AB
cargo run -- --latitude 51.0447 --longitude=-114.0719

# Analyze with more stations (10 instead of default 5)
cargo run -- --latitude 49.8951 --longitude=-97.1384 -n 10
```

This will:
1. Find the nearest weather stations to the specified coordinates
2. Analyze climate data from all stations (default: 5 stations)
3. Calculate and display the average optimal tire change dates

### Example Locations

Some example Canadian cities you can try:

```bash
# Windsor, ON
cargo run -- --latitude 42.3149 --longitude=-83.0364

# London, ON
cargo run -- --latitude 42.9849 --longitude=-81.2453

# Thunder Bay, ON
cargo run -- --latitude 48.3809 --longitude=-89.2477

# Winnipeg, MB
cargo run -- --latitude 49.8951 --longitude=-97.1384

# Calgary, AB
cargo run -- --latitude 51.0447 --longitude=-114.0719
```

**Note**: For negative longitudes (west of prime meridian), use `--longitude=-VALUE` format with the equals sign.

## Command Line Options

```
Options:
      --serve                        Run as API server
      --port <PORT>                  Port to run the API server on [default: 3000]
      --db-path <DB_PATH>            Database file path [default: tireswap.db]
      --update-db                    Update the database with latest weather station and climate data
      --latitude <LATITUDE>          Latitude of the location to analyze
      --longitude <LONGITUDE>        Longitude of the location to analyze
  -n, --num-stations <NUM_STATIONS>  Number of nearest stations to consider for analysis [default: 5]
  -h, --help                         Print help
```

### Options Details

- **`--serve`**: Run the application as a REST API server
- **`--port`**: Specify the port for the API server (default: 3000)
- **`--db-path`**: Path to the SQLite database file (default: tireswap.db)
- **`--update-db`**: Fetches and stores weather station and climate data. Run this once initially, or periodically to refresh data.
- **`--latitude`**: Latitude coordinate of your location (decimal degrees) - **Required** for CLI analysis
- **`--longitude`**: Longitude coordinate of your location (decimal degrees, negative for western hemisphere) - **Required** for CLI analysis
- **`-n, --num-stations`**: How many nearby stations to include in the analysis (more stations = broader regional average)

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
├── API.md                  # REST API documentation
├── test_api.sh            # API testing script
├── src/
│   ├── main.rs            # Main entry point, CLI interface, and server setup
│   ├── api.rs             # REST API routes and handlers
│   ├── aggregator.rs      # Data fetching from Environment Canada API
│   ├── db.rs              # Database operations and schema (thread-safe)
│   ├── nearest.rs         # KD-tree spatial search for finding nearest stations
│   └── analyzer.rs        # Tire swap recommendation analyzer
└── tireswap.db           # SQLite database (created on first run)
```

## Modules

### `api`
Provides REST API endpoints using the Axum web framework. Handles HTTP requests for health checks and tire swap recommendations.

### `analyzer`
Provides the `Analyzer` struct which takes a location (latitude/longitude) and calculates optimal tire change dates by:
- Finding the k-nearest weather stations
- Collecting climate data from each station
- Computing average dates across all stations

### `aggregator`
Handles all API communication with Environment Canada to fetch station lists and climate data.

### `db`
Manages SQLite database operations including schema initialization and CRUD operations for stations and climate data.

### `nearest`
Implements efficient spatial search using KD-tree data structure to quickly find closest weather stations to any location.

## Dependencies
See [Cargo.toml](Cargo.toml) for the complete list.
