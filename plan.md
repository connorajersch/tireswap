# TireSwap Project Plan

## Project Goal
Build a public web service that recommends optimal dates for switching between summer and winter tires in Canada, using historical climate data from Environment and Climate Change Canada (ECCC). The experience should be fast, reliable, and easy to use, with no private or user-specific data.

## Product Scope (MVP)
- Search by city or postal code to find a location.
- Show recommended switch dates based on nearby climate stations.
- Explain the recommendation with supporting data (e.g., recent-year averages).
- Mobile-friendly UI.

## Current Backend Capabilities
- Rust backend with Axum REST API.
- SQLite database with station and climate data.
- K-nearest station selection via KD-tree.
- Analyzer that computes optimal tire swap dates.
- API endpoint: `GET /api/optimal-dates?latitude={lat}&longitude={lon}&num_stations={n}`.

## Frontend Responsibilities
- Location search (city/postal code).
- Call the backend with lat/lon.
- Render recommendations and confidence indicators.
- Show data quality notes (e.g., number of stations, years of history).
- Handle errors gracefully and show helpful guidance.

## Backend Responsibilities
- Maintain station and climate data in SQLite.
- Compute recommendations efficiently and consistently.
- Provide a stable REST API contract.
- Support rate limiting by client token (public data only).

## API Contract (Initial)
- `GET /health`
- `GET /api/optimal-dates?latitude={lat}&longitude={lon}&num_stations={n}`

Planned additions:
- `GET /api/search?query={city_or_postal}` -> returns coordinates and location metadata.
- `GET /api/stations/nearby?latitude={lat}&longitude={lon}&num_stations={n}` -> optional debugging or transparency.

## Data Flow
1. User enters city/postal code.
2. Frontend resolves to coordinates (via backend search or external geocoding).
3. Frontend calls `/api/optimal-dates` with lat/lon.
4. Backend selects nearest stations, analyzes climate data, returns dates + metadata.
5. Frontend renders results and explanation.

## Scalability Strategy
- Read-heavy endpoints only.
- Cache results by rounded lat/lon + `num_stations` (in-memory or Redis later).
- Precompute results for common population centers.
- Use CDN caching for public GET responses.
- Add indexes for common query paths in SQLite (or migrate to Postgres later if needed).

## Rate Limiting (Public Tokens)
- Per-client token in `Authorization: Bearer <token>` header.
- In-memory rate limiter keyed by token for single-instance.
- Upgrade to Redis when deploying multiple instances.

## Observability
- Log request ID, client token ID, latency, and status.
- Track rate limit hits and error rates.
- Add basic uptime checks on `/health`.

## Deployment and Environments
- Local: developer machine with SQLite.
- Staging: seeded database and limited tokens.
- Production: scheduled data refresh and caching.

## Roadmap
1. Implement location search (postal code + city).
2. Finalize API response format with metadata and confidence info.
3. Build React + Vite frontend.
4. Add caching and rate limiting.
5. Improve visualization (trend charts and station coverage).
6. Optional: background jobs to refresh data on a schedule.

## Open Questions
- Exact API response schema for optimal dates and confidence metrics.
- Best approach for geocoding (backend or frontend).
- Recommended caching granularity for lat/lon.
