# Tire Swap API Documentation

## Overview

The Tire Swap API provides endpoints to get optimal tire swap dates based on weather station data and climate analysis for any location.

## Base URL

When running locally: `http://localhost:3000`

## Starting the Server

```bash
# Start the API server on default port (3000)
cargo run -- --serve

# Start on a custom port
cargo run -- --serve --port 8080

# Use a custom database file
cargo run -- --serve --db-path /path/to/custom.db
```

## Endpoints

### Health Check

Check if the API server is running.

**Endpoint:** `GET /health`

**Response:**
```json
{
  "status": "ok",
  "service": "tireswap-api"
}
```

**Example:**
```bash
curl http://localhost:3000/health
```

---

### Get Optimal Tire Swap Dates

Get recommended tire swap dates for a specific location based on historical weather data from nearby weather stations.

**Endpoint:** `GET /api/optimal-dates`

**Query Parameters:**

| Parameter | Type | Required | Default | Description |
|-----------|------|----------|---------|-------------|
| `latitude` | float | Yes | - | Latitude of the location (-90 to 90) |
| `longitude` | float | Yes | - | Longitude of the location (-180 to 180) |
| `num_stations` | integer | No | 5 | Number of nearest weather stations to analyze (1-20 recommended) |

**Response:**

```json
{
  "latitude": 43.7,
  "longitude": -79.4,
  "switch_to_summer": "April 15",
  "switch_to_winter": "October 25",
  "stations_analyzed": 5
}
```

**Response Fields:**

- `latitude`: The latitude of the queried location
- `longitude`: The longitude of the queried location
- `switch_to_summer`: Recommended date to switch to summer tires (null if no data available)
- `switch_to_winter`: Recommended date to switch to winter tires (null if no data available)
- `stations_analyzed`: Number of weather stations used in the analysis

**Example Requests:**

```bash
# Toronto, Ontario
curl "http://localhost:3000/api/optimal-dates?latitude=43.7&longitude=-79.4"

# Vancouver, BC (using 10 stations)
curl "http://localhost:3000/api/optimal-dates?latitude=49.28&longitude=-123.12&num_stations=10"

# Montreal, Quebec
curl "http://localhost:3000/api/optimal-dates?latitude=45.5&longitude=-73.6"
```

**Error Response:**

If an error occurs, the API returns an error response:

```json
{
  "error": "Error message describing what went wrong"
}
```

**HTTP Status Codes:**

- `200 OK`: Successful request
- `500 Internal Server Error`: Server error (e.g., database error, analysis failure)

## Data Requirements

Before using the API, you need to populate the database with weather station and climate data:

```bash
# Update the database with latest weather station and climate data
cargo run -- --update-db

# Use a custom database file
cargo run -- --update-db --db-path /path/to/custom.db
```

This process:
1. Fetches weather station data from the API
2. Downloads historical climate data for each station
3. Analyzes the data to determine optimal tire swap dates

**Note:** The update process can take several minutes to hours depending on the number of weather stations.

## CLI Mode

The backend also supports command-line usage for one-off queries:

```bash
# Analyze a specific location
cargo run -- --latitude 43.7 --longitude -79.4

# Analyze using more stations
cargo run -- --latitude 43.7 --longitude -79.4 --num-stations 10
```

## Architecture

The API is built using:
- **Axum**: Modern, ergonomic web framework for Rust
- **Tokio**: Async runtime for handling concurrent requests
- **SQLite**: Embedded database for storing weather and climate data
- **Kiddo**: K-d tree implementation for efficient nearest neighbor search

## Thread Safety

The database connection is protected by a `Mutex` to ensure thread-safe access across multiple concurrent API requests.
