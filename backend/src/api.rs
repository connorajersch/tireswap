use axum::{
    extract::{Query, State},
    http::StatusCode,
    response::{IntoResponse, Json},
    routing::get,
    Router,
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;

use crate::analyzer::{Analyzer, Recommendation};
use crate::db::Database;

/// Application state shared across all handlers
#[derive(Clone)]
pub struct AppState {
    pub db: Arc<Database>,
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
        .with_state(state)
}
