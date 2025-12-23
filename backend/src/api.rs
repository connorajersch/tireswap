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
}

impl From<Recommendation> for OptimalDatesResponse {
    fn from(rec: Recommendation) -> Self {
        Self {
            latitude: rec.latitude,
            longitude: rec.longitude,
            switch_to_summer: rec.switch_to_summer,
            switch_to_winter: rec.switch_to_winter,
            stations_analyzed: rec.stations_analyzed,
        }
    }
}

/// Error response body
#[derive(Debug, Serialize)]
pub struct ErrorResponse {
    pub error: String,
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
    // Create analyzer
    let analyzer = Analyzer::new(&state.db).map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse {
                error: format!("Failed to create analyzer: {}", e),
            }),
        )
    })?;

    // Analyze the location
    let recommendation = analyzer
        .analyze(query.latitude, query.longitude, query.num_stations)
        .map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorResponse {
                    error: format!("Analysis failed: {}", e),
                }),
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
