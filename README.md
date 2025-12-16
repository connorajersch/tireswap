# TireSwap 🚗❄️

A web service that helps users determine the optimal date to switch between summer and winter tires based on historical weather data from the Government of Canada.

## Overview

TireSwap analyzes historical weather data to determine the average date of first and last snowfall, as well as temperature patterns (specifically when temperatures are above or below 7°C). This information helps users make informed decisions about when to switch their tires for optimal safety and compliance with local regulations.

## Features

- **Historical Weather Data Analysis**: Leverages the Government of Canada's free weather API
- **Climate Station Database**: Stores and manages data from climate stations across Canada
- **Optimal Timing Recommendations**: Calculates the best dates for tire switching based on multi-year trends
- **SQLite Storage**: Fast data retrieval with local database storage

## Technology Stack

- **Language**: Rust
- **Database**: SQLite (with bundled support)
- **HTTP Client**: reqwest (async with tokio)
- **Data Format**: JSON & CSV

## Project Structure

```
tireswap/
├── backend/
│   ├── Cargo.toml          # Rust dependencies and project metadata
│   └── src/
│       ├── main.rs         # Application entry point
│       ├── aggregator/     # Weather data aggregation logic
│       │   ├── mod.rs
│       │   └── aggregator.rs
│       └── db/             # Database operations
│           ├── mod.rs
│           └── db.rs
└── README.md
```

## Getting Started

### Prerequisites

- Rust (edition 2024)
- Cargo

### Installation

1. Clone the repository:
```bash
git clone https://github.com/yourusername/tireswap.git
cd tireswap
```

2. Navigate to the backend directory:
```bash
cd backend
```

3. Build the project:
```bash
cargo build
```

4. Run the application:
```bash
cargo run
```

## Dependencies

- **reqwest** (v0.12) - HTTP client with blocking and JSON features
- **tokio** (v1) - Async runtime with full features
- **rusqlite** (v0.37) - SQLite bindings with bundled SQLite
- **serde_json** (v1.0) - JSON serialization/deserialization

## How It Works

1. **Data Collection**: The aggregator fetches climate station data from the Government of Canada's API
2. **Data Storage**: Station information (ID, name, coordinates) is stored in a local SQLite database
3. **Analysis**: Historical weather data is processed to identify patterns in snowfall and temperature
4. **Recommendations**: Based on the analyzed data, optimal tire switching dates are calculated

## Data Sources

This project uses the [Government of Canada Climate Data API](https://api.weather.gc.ca/):
- Climate stations: `https://api.weather.gc.ca/collections/climate-stations/items`
- Historical weather data: `https://climate.weather.gc.ca/climate_data/bulk_data_e.html`

## Development Status

This project is currently in active development. Current functionality includes:
- 🚧 Climate station data fetching and storage
- 🚧 Historical weather data analysis
- 🚧 Tire switching recommendation engine
- 🚧 API server implementation

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request.

## License

This project is open source and available under the MIT License.

## Acknowledgments

- Weather data provided by the [Government of Canada](https://weather.gc.ca/)
- Based on the principle that winter tires perform better below 7°C while summer tires are optimal above that threshold
