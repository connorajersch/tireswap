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
  "stations_analyzed": 5,
  "stations": {
    "requested": 5,
    "returned": 5,
    "list": [
      { "id": 4607, "name": "TORONTO CITY", "distance_km": 3.2 }
    ],
    "distance_km": { "min": 3.2, "avg": 12.8, "max": 25.4 }
  },
  "quality": {
    "summer": {
      "stations_with_data": 4,
      "coverage_pct": 80.0
    },
    "winter": {
      "stations_with_data": 5,
      "coverage_pct": 100.0
    },
    "data_years": {
      "min_span_years": 8,
      "avg_span_years": 12.4,
      "max_span_years": 19
    }
  }
}
```

**Response Fields:**

- `latitude`: The latitude of the queried location
- `longitude`: The longitude of the queried location
- `switch_to_summer`: Recommended date to switch to summer tires (null if no data available)
- `switch_to_winter`: Recommended date to switch to winter tires (null if no data available)
- `stations_analyzed`: Number of weather stations used in the analysis
- `stations`: Station metadata including list and distance summary
- `quality`: Coverage and data-quality metrics

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
  "error": {
    "code": "ANALYSIS_FAILED",
    "message": "Analysis failed",
    "details": "Optional low-level error details"
  }
}
```

**HTTP Status Codes:**

- `200 OK`: Successful request
- `400 Bad Request`: Invalid query parameters
- `500 Internal Server Error`: Server error (e.g., database error, analysis failure)

**Error Codes:**

- `INVALID_QUERY`: Invalid query parameters
- `ANALYSIS_FAILED`: Analyzer or data processing failure
- `INTERNAL_ERROR`: Unexpected internal error

---

### Search For City Or Canadian Postal Code

Resolve a city name or Canadian postal code to coordinates and basic location metadata.

**Endpoint:** `GET /api/search`

**Query Parameters:**

| Parameter | Type | Required | Default | Description |
|-----------|------|----------|---------|-------------|
| `query` | string | Yes | - | City name or Canadian postal code (e.g., `Toronto`, `M5V 2T6`) |

**Response:**

```json
{
  "query": "Toronto",
  "normalized_query": "Toronto",
  "results": [
    {
      "city": "Toronto",
      "province": "Ontario",
      "postal_code": "M5V 2T6",
      "lat": 43.653226,
      "lon": -79.3831843,
      "source": "nominatim"
    }
  ]
}
```

**Response Fields:**

- `query`: The original query string
- `normalized_query`: Normalized query string used for lookup
- `results`: List of location matches (currently returns the top match)
- `city`: City or municipality name (when available)
- `province`: Province/territory name (when available)
- `postal_code`: Postal code if returned by the provider
- `lat`: Latitude
- `lon`: Longitude
- `source`: Geocoding provider identifier

**Example Requests:**

```bash
# City lookup
curl "http://localhost:3000/api/search?query=Toronto"

# Postal code lookup
curl "http://localhost:3000/api/search?query=M5V%202T6"
```

**Error Response:**

```json
{
  "error": {
    "code": "NOT_FOUND",
    "message": "No results found for query",
    "details": "Try a different city or Canadian postal code"
  }
}
```

**HTTP Status Codes:**

- `200 OK`: Successful request
- `400 Bad Request`: Invalid query (missing/empty)
- `404 Not Found`: No matching location
- `502 Bad Gateway`: Upstream geocoder error

**Error Codes:**

- `INVALID_QUERY`: Missing/empty query parameter
- `NOT_FOUND`: No results for the query
- `GEOCODE_FAILED`: Upstream geocoder request failure

**Provider Notes:**

- Geocoding uses OpenStreetMap Nominatim. Respect their usage policy and rate limits (1 request/second).
- The API caches successful results in memory for 1 hour to reduce upstream calls.
- Set `TIRESWAP_NOMINATIM_UA` to customize the User-Agent header for Nominatim requests.
