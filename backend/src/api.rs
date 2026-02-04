use axum::{
    extract::{Query, State},
    http::StatusCode,
    response::{IntoResponse, Json},
    routing::get,
    Router,
};
use serde::{Deserialize, Serialize};
use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
    time::{Duration, Instant},
};

use crate::analyzer::{Analyzer, Recommendation};
use crate::db::Database;

const GEOCODE_CACHE_TTL: Duration = Duration::from_secs(60 * 60);

/// Application state shared across all handlers
#[derive(Clone)]
pub struct AppState {
    pub db: Arc<Database>,
    pub geocode_client: reqwest::Client,
    pub geocode_cache: Arc<Mutex<HashMap<String, CacheEntry>>>,
    pub geocode_base_url: String,
}

/// Query parameters for the optimal dates endpoint
#[derive(Debug, Deserialize)]
pub struct OptimalDatesQuery {
    /// Latitude of the location
    latitude: f64,
    /// Longitude of the location
    longitude: f64,
    /// Number of nearest stations to consider (default: 5)
    #[serde(default = "default_num_stations")]
    num_stations: usize,
}

fn default_num_stations() -> usize {
    5
}

/// Response body for optimal dates
#[derive(Debug, Serialize)]
pub struct OptimalDatesResponse {
    pub latitude: f64,
    pub longitude: f64,
    pub switch_to_summer: Option<String>,
    pub switch_to_winter: Option<String>,
    pub stations_analyzed: usize,
    pub stations: StationMeta,
    pub quality: QualitySummary,
}

impl From<Recommendation> for OptimalDatesResponse {
    fn from(rec: Recommendation) -> Self {
        let station_list: Vec<StationSummary> = rec
            .stations
            .iter()
            .map(|station| StationSummary {
                id: station.id,
                name: station.name.clone(),
                distance_km: station.distance_km,
            })
            .collect();

        let distance_km = calculate_distance_summary(&rec.stations);
        let stations_returned = rec.stations.len();
        let summer_coverage = calculate_coverage_pct(rec.summer_stations_with_data, stations_returned);
        let winter_coverage = calculate_coverage_pct(rec.winter_stations_with_data, stations_returned);

        Self {
            latitude: rec.latitude,
            longitude: rec.longitude,
            switch_to_summer: rec.switch_to_summer,
            switch_to_winter: rec.switch_to_winter,
            stations_analyzed: rec.stations_analyzed,
            stations: StationMeta {
                requested: rec.stations_requested,
                returned: stations_returned,
                list: station_list,
                distance_km,
            },
            quality: QualitySummary {
                summer: SeasonalQuality {
                    stations_with_data: rec.summer_stations_with_data,
                    coverage_pct: summer_coverage,
                },
                winter: SeasonalQuality {
                    stations_with_data: rec.winter_stations_with_data,
                    coverage_pct: winter_coverage,
                },
                data_years: DataYearsSummary {
                    min_span_years: rec.data_years.min_span_years,
                    avg_span_years: rec.data_years.avg_span_years,
                    max_span_years: rec.data_years.max_span_years,
                },
            },
        }
    }
}

/// Error response body
#[derive(Debug, Serialize)]
pub struct ErrorResponse {
    pub error: ErrorBody,
}

#[derive(Debug, Serialize)]
pub struct ErrorBody {
    pub code: String,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub details: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct SearchQuery {
    pub query: String,
}

#[derive(Debug, Serialize, Clone)]
pub struct SearchResponse {
    pub query: String,
    pub normalized_query: String,
    pub results: Vec<LocationResult>,
}

#[derive(Debug, Serialize, Clone)]
pub struct LocationResult {
    pub city: Option<String>,
    pub province: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub postal_code: Option<String>,
    pub lat: f64,
    pub lon: f64,
    pub source: String,
}

#[derive(Debug, Clone)]
pub struct CacheEntry {
    value: SearchResponse,
    cached_at: Instant,
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum SearchKind {
    PostalCode { normalized: String },
    City { normalized: String },
}

#[derive(Debug, Serialize)]
pub struct StationMeta {
    pub requested: usize,
    pub returned: usize,
    pub list: Vec<StationSummary>,
    pub distance_km: DistanceSummary,
}

#[derive(Debug, Serialize)]
pub struct StationSummary {
    pub id: i64,
    pub name: String,
    pub distance_km: f64,
}

#[derive(Debug, Serialize)]
pub struct DistanceSummary {
    pub min: Option<f64>,
    pub avg: Option<f64>,
    pub max: Option<f64>,
}

#[derive(Debug, Serialize)]
pub struct QualitySummary {
    pub summer: SeasonalQuality,
    pub winter: SeasonalQuality,
    pub data_years: DataYearsSummary,
}

#[derive(Debug, Serialize)]
pub struct SeasonalQuality {
    pub stations_with_data: usize,
    pub coverage_pct: f64,
}

#[derive(Debug, Serialize)]
pub struct DataYearsSummary {
    pub min_span_years: Option<i64>,
    pub avg_span_years: Option<f64>,
    pub max_span_years: Option<i64>,
}

fn calculate_distance_summary(stations: &[crate::nearest::StationWithDistance]) -> DistanceSummary {
    if stations.is_empty() {
        return DistanceSummary {
            min: None,
            avg: None,
            max: None,
        };
    }

    let mut min = f64::INFINITY;
    let mut max = 0.0;
    let mut sum = 0.0;

    for station in stations {
        let distance = station.distance_km;
        if distance < min {
            min = distance;
        }
        if distance > max {
            max = distance;
        }
        sum += distance;
    }

    DistanceSummary {
        min: Some(min),
        avg: Some(sum / stations.len() as f64),
        max: Some(max),
    }
}

fn calculate_coverage_pct(stations_with_data: usize, stations_returned: usize) -> f64 {
    if stations_returned == 0 {
        0.0
    } else {
        (stations_with_data as f64 / stations_returned as f64) * 100.0
    }
}

fn error_response(
    status: StatusCode,
    code: &str,
    message: &str,
    details: Option<String>,
) -> (StatusCode, Json<ErrorResponse>) {
    (
        status,
        Json(ErrorResponse {
            error: ErrorBody {
                code: code.to_string(),
                message: message.to_string(),
                details,
            },
        }),
    )
}

fn parse_search_query(raw: &str) -> Result<SearchKind, String> {
    let trimmed = raw.trim();
    if trimmed.is_empty() {
        return Err("query must not be empty".to_string());
    }

    if let Some(normalized) = normalize_postal_code(trimmed) {
        return Ok(SearchKind::PostalCode { normalized });
    }

    let normalized = trimmed
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ");

    Ok(SearchKind::City { normalized })
}

fn normalize_postal_code(raw: &str) -> Option<String> {
    let compact: String = raw
        .chars()
        .filter(|c| c.is_ascii_alphanumeric())
        .map(|c| c.to_ascii_uppercase())
        .collect();

    if compact.len() != 3 && compact.len() != 6 {
        return None;
    }

    let chars: Vec<char> = compact.chars().collect();
    if !is_postal_letter(chars[0], true) || !chars[1].is_ascii_digit() || !is_postal_letter(chars[2], false) {
        return None;
    }

    if compact.len() == 6 {
        if !chars[3].is_ascii_digit()
            || !is_postal_letter(chars[4], false)
            || !chars[5].is_ascii_digit()
        {
            return None;
        }
        return Some(format!("{} {}",
            compact[0..3].to_string(),
            compact[3..6].to_string()
        ));
    }

    Some(compact)
}

fn is_postal_letter(letter: char, is_first: bool) -> bool {
    let allowed_first = "ABCEGHJKLMNPRSTVXY";
    let allowed_other = "ABCEGHJKLMNPRSTVWXYZ";
    if !letter.is_ascii_uppercase() {
        return false;
    }
    if is_first {
        allowed_first.contains(letter)
    } else {
        allowed_other.contains(letter)
    }
}

fn cache_key(kind: &SearchKind) -> String {
    match kind {
        SearchKind::PostalCode { normalized } => format!("postal:{}", normalized.to_lowercase()),
        SearchKind::City { normalized } => format!("city:{}", normalized.to_lowercase()),
    }
}

async fn geocode_with_nominatim(
    client: &reqwest::Client,
    base_url: &str,
    kind: &SearchKind,
) -> Result<Vec<LocationResult>, String> {
    let query = match kind {
        SearchKind::PostalCode { normalized } => format!("{}, Canada", normalized),
        SearchKind::City { normalized } => format!("{}, Canada", normalized),
    };

    let encoded = urlencoding::encode(&query);
    let base = base_url.trim_end_matches('/');
    let url = format!(
        "{}/search?q={}&format=json&addressdetails=1&limit=1&countrycodes=ca",
        base, encoded
    );

    let response = client
        .get(url)
        .send()
        .await
        .map_err(|e| format!("geocoding request failed: {}", e))?;

    if !response.status().is_success() {
        return Err(format!("geocoding request failed with status {}", response.status()));
    }

    let results: Vec<NominatimResult> = response
        .json()
        .await
        .map_err(|e| format!("failed to parse geocoding response: {}", e))?;

    let mut locations = Vec::new();
    for result in results.into_iter().take(1) {
        let (lat, lon) = match (result.lat.parse::<f64>(), result.lon.parse::<f64>()) {
            (Ok(lat), Ok(lon)) => (lat, lon),
            _ => continue,
        };

        let (city, province, postal_code) = if let Some(address) = result.address {
            let city = address
                .city
                .or(address.town)
                .or(address.village)
                .or(address.municipality)
                .or(address.county);
            let province = address.state.or(address.province);
            (city, province, address.postcode)
        } else {
            (None, None, None)
        };

        locations.push(LocationResult {
            city,
            province,
            postal_code,
            lat,
            lon,
            source: "nominatim".to_string(),
        });
    }

    Ok(locations)
}

/// Handler for GET /api/optimal-dates
/// 
/// Returns optimal tire swap dates for a given location
/// 
/// Query parameters:
/// - latitude: f64 (required)
/// - longitude: f64 (required)
/// - num_stations: usize (optional, default: 5)
async fn get_optimal_dates(
    State(state): State<AppState>,
    Query(query): Query<OptimalDatesQuery>,
) -> Result<Json<OptimalDatesResponse>, (StatusCode, Json<ErrorResponse>)> {
    let mut violations = Vec::new();
    if !(-90.0..=90.0).contains(&query.latitude) {
        violations.push("latitude must be between -90 and 90".to_string());
    }
    if !(-180.0..=180.0).contains(&query.longitude) {
        violations.push("longitude must be between -180 and 180".to_string());
    }
    if !(1..=20).contains(&query.num_stations) {
        violations.push("num_stations must be between 1 and 20".to_string());
    }
    if !violations.is_empty() {
        return Err(error_response(
            StatusCode::BAD_REQUEST,
            "INVALID_QUERY",
            "Invalid query parameters",
            Some(violations.join("; ")),
        ));
    }

    // Create analyzer
    let analyzer = Analyzer::new(&state.db).map_err(|e| {
        error_response(
            StatusCode::INTERNAL_SERVER_ERROR,
            "INTERNAL_ERROR",
            "Failed to create analyzer",
            Some(e.to_string()),
        )
    })?;

    // Analyze the location
    let recommendation = analyzer
        .analyze(query.latitude, query.longitude, query.num_stations)
        .map_err(|e| {
            error_response(
                StatusCode::INTERNAL_SERVER_ERROR,
                "ANALYSIS_FAILED",
                "Analysis failed",
                Some(e.to_string()),
            )
        })?;

    Ok(Json(recommendation.into()))
}

/// Handler for GET /api/search
///
/// Returns coordinates and location metadata for a city or Canadian postal code
async fn get_search(
    State(state): State<AppState>,
    Query(query): Query<SearchQuery>,
) -> Result<Json<SearchResponse>, (StatusCode, Json<ErrorResponse>)> {
    let kind = parse_search_query(&query.query).map_err(|details| {
        error_response(
            StatusCode::BAD_REQUEST,
            "INVALID_QUERY",
            "Invalid query parameters",
            Some(details),
        )
    })?;

    let normalized_query = match &kind {
        SearchKind::PostalCode { normalized } => normalized.clone(),
        SearchKind::City { normalized } => normalized.clone(),
    };

    let cache_key = cache_key(&kind);
    if let Ok(cache) = state.geocode_cache.lock() {
        if let Some(entry) = cache.get(&cache_key) {
            if entry.cached_at.elapsed() < GEOCODE_CACHE_TTL {
                return Ok(Json(entry.value.clone()));
            }
        }
    }

    let results = geocode_with_nominatim(
        &state.geocode_client,
        &state.geocode_base_url,
        &kind,
    )
        .await
        .map_err(|details| {
            error_response(
                StatusCode::BAD_GATEWAY,
                "GEOCODE_FAILED",
                "Geocoding request failed",
                Some(details),
            )
        })?;

    if results.is_empty() {
        return Err(error_response(
            StatusCode::NOT_FOUND,
            "NOT_FOUND",
            "No results found for query",
            Some("Try a different city or Canadian postal code".to_string()),
        ));
    }

    let response = SearchResponse {
        query: query.query,
        normalized_query,
        results,
    };

    if let Ok(mut cache) = state.geocode_cache.lock() {
        cache.insert(
            cache_key,
            CacheEntry {
                value: response.clone(),
                cached_at: Instant::now(),
            },
        );
    }

    Ok(Json(response))
}

/// Health check endpoint
async fn health_check() -> impl IntoResponse {
    Json(serde_json::json!({
        "status": "ok",
        "service": "tireswap-api"
    }))
}

/// Create the API router with all routes
pub fn create_router(state: AppState) -> Router {
    Router::new()
        .route("/health", get(health_check))
        .route("/api/optimal-dates", get(get_optimal_dates))
        .route("/api/search", get(get_search))
        .with_state(state)
}

#[derive(Debug, Deserialize)]
struct NominatimResult {
    lat: String,
    lon: String,
    #[serde(default)]
    address: Option<NominatimAddress>,
}

#[derive(Debug, Deserialize)]
struct NominatimAddress {
    #[serde(default)]
    city: Option<String>,
    #[serde(default)]
    town: Option<String>,
    #[serde(default)]
    village: Option<String>,
    #[serde(default)]
    municipality: Option<String>,
    #[serde(default)]
    county: Option<String>,
    #[serde(default)]
    state: Option<String>,
    #[serde(default)]
    province: Option<String>,
    #[serde(default)]
    postcode: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_search_query_city() {
        let kind = parse_search_query("  Toronto  ").unwrap();
        assert_eq!(
            kind,
            SearchKind::City {
                normalized: "Toronto".to_string()
            }
        );
    }

    #[test]
    fn parse_search_query_postal_full() {
        let kind = parse_search_query("M5V 2T6").unwrap();
        assert_eq!(
            kind,
            SearchKind::PostalCode {
                normalized: "M5V 2T6".to_string()
            }
        );
    }

    #[test]
    fn parse_search_query_postal_fsa() {
        let kind = parse_search_query("h2b").unwrap();
        assert_eq!(
            kind,
            SearchKind::PostalCode {
                normalized: "H2B".to_string()
            }
        );
    }

    #[test]
    fn parse_search_query_empty() {
        let err = parse_search_query("   ").unwrap_err();
        assert_eq!(err, "query must not be empty");
    }
}
