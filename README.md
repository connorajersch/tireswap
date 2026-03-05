# TireSwap 🚗❄️

A smart web service that helps Canadian drivers determine the optimal dates to switch between summer and winter tires based on decades of historical weather data.

## Why TireSwap?

Every year, Canadian drivers face the same question: when should I switch my tires? Switch too early and you waste money on unnecessary wear. Switch too late and you risk your safety on icy roads—and potential fines in provinces with mandatory winter tire regulations.

TireSwap solves this problem by analyzing years of historical weather patterns from your specific location to provide data-driven recommendations. No more guessing, no more relying on arbitrary calendar dates. Just smart, localized advice based on real climate trends.

## What We Use

Winter tires are engineered to perform better below 7°C, while summer tires excel above that threshold. TireSwap analyzes:

- **Temperature Patterns**: When does your area consistently cross the 7°C threshold?
- **Multi-Year Trends**: Statistical analysis across recent years of data for reliable predictions
- **Local Climate Stations**: Uses the nearest weather stations for local results

## How It Works

1. **Find Your Location**: Enter your city or postal code
2. **Get Personalized Dates**: Receive optimal switch dates based on historical data from nearby climate stations
3. **Plan Ahead**: Schedule your tire appointments with confidence, knowing you're making a data-informed decision

## Project Components

TireSwap consists of two main components:

- **Backend**: Data aggregation, analysis engine, and REST API (see [backend/README.md](backend/README.md))
- **Frontend**: React + Vite user interface scaffold (see [frontend/README.md](frontend/README.md))

## Data Sources

TireSwap uses official data from:
- [Government of Canada Climate Data API](https://api.weather.gc.ca/)
- Historical weather observations from Environment and Climate Change Canada

All weather data is freely available and publicly accessible, ensuring transparency and reproducibility.

## Getting Started

This project is organized as a monorepo:

```
tireswap/
├── backend/     # Data processing and API server
├── frontend/    # Web application
└── README.md    # This file
```

For setup and development instructions, see the README in each component directory.

### Frontend Quickstart

```bash
cd frontend
npm install
npm run dev
```

Create a `frontend/.env.local` file with:

```bash
VITE_API_BASE_URL=http://localhost:8080
VITE_API_TOKEN=replace-me
```

## Build, Deploy, and Local Debug Tooling

The repository includes script-first tooling for local debug and VPS deployment:

```bash
# build backend + frontend production artifacts
make build

# package a release tarball into dist/
make package RELEASE=20260305153000

# deploy on a VPS (run on the VPS host)
make deploy RELEASE=20260305153000

# rollback on a VPS (run on the VPS host)
make rollback RELEASE=20260304120000

# run backend + frontend together for local debugging
make debug
```

Script entrypoints:

- `scripts/build_backend.sh`
- `scripts/build_frontend.sh`
- `scripts/package_release.sh <release_id>`
- `scripts/deploy_vps.sh <release_id>`
- `scripts/rollback_vps.sh <release_id>`
- `scripts/debug_local.sh [--update-db-first]`

Deployment templates are in `deploy/systemd/` and `deploy/nginx/`.

### GitHub Actions CI/CD

`.github/workflows/ci-deploy.yml` runs tests/build/package on `main` and can deploy to a VPS over SSH.

Set these repository secrets to enable deployment:

- `VPS_HOST`
- `VPS_USER`
- `VPS_SSH_KEY`
- `VPS_SSH_PORT` (optional, defaults to `22`)

## Development Roadmap

- ✅ Climate station data collection infrastructure
- ✅ Historical weather data analysis engine
- ✅ Recommendation algorithm (temperature-based)
- ✅ REST API server with location-based queries
- ✅ Nearest station selection algorithm
- ✅ Web frontend scaffold
- 📋 Postal code and city name search
- 📋 Multi-year trend visualization
- 📋 Mobile-responsive design
- 📋 Confidence intervals and data quality indicators

## License

This project is open source and available under the MIT License.

## Acknowledgments

- Weather data provided by [Environment and Climate Change Canada](https://weather.gc.ca/)
