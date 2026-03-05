use axum::{body::Body, http::Request};
use httpmock::MockServer;
use serde_json::Value;
use tower::util::ServiceExt;

use backend::api::{AppState, create_router};
use backend::db::Database;

fn build_state(base_url: String) -> AppState {
    let db = Database::new_in_memory().expect("db init");
    db.initialize_schema().expect("schema init");

    AppState {
        db: std::sync::Arc::new(db),
        geocode_client: reqwest::Client::new(),
        geocode_cache: std::sync::Arc::new(std::sync::Mutex::new(std::collections::HashMap::new())),
        geocode_base_url: base_url,
        geocode_api_key: Some("test-key".to_string()),
    }
}

#[tokio::test]
async fn search_city_success() {
    let server = MockServer::start();
    let _mock = server.mock(|when, then| {
        when.method("GET")
            .path("/maps/api/geocode/json")
            .query_param("address", "Toronto, Canada")
            .query_param("components", "country:CA")
            .query_param("region", "ca")
            .query_param("key", "test-key");
        then.status(200)
            .header("content-type", "application/json")
            .body(
                r#"{
                    "status": "OK",
                    "results": [{
                        "geometry": { "location": { "lat": 43.653226, "lng": -79.3831843 } },
                        "address_components": [
                            { "long_name": "Toronto", "types": ["locality", "political"] },
                            { "long_name": "Ontario", "types": ["administrative_area_level_1", "political"] },
                            { "long_name": "M5V 2T6", "types": ["postal_code"] }
                        ]
                    }]
                }"#,
            );
    });

    let app = create_router(build_state(server.url("")));
    let response = app
        .oneshot(
            Request::builder()
                .uri("/api/search?query=Toronto")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), 200);
    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let json: Value = serde_json::from_slice(&body).unwrap();

    assert_eq!(json["normalized_query"], "Toronto");
    assert_eq!(json["results"][0]["city"], "Toronto");
    assert_eq!(json["results"][0]["province"], "Ontario");
    assert_eq!(json["results"][0]["postal_code"], "M5V 2T6");
    assert_eq!(json["results"][0]["source"], "google_maps");
}

#[tokio::test]
async fn search_postal_success() {
    let server = MockServer::start();
    let _mock = server.mock(|when, then| {
        when.method("GET")
            .path("/maps/api/geocode/json")
            .query_param("address", "M5V 2T6, Canada")
            .query_param("components", "country:CA|postal_code:M5V2T6")
            .query_param("region", "ca")
            .query_param("key", "test-key");
        then.status(200)
            .header("content-type", "application/json")
            .body(
                r#"{
                    "status": "OK",
                    "results": [{
                        "geometry": { "location": { "lat": 43.651070, "lng": -79.347015 } },
                        "address_components": [
                            { "long_name": "Toronto", "types": ["locality", "political"] },
                            { "long_name": "Ontario", "types": ["administrative_area_level_1", "political"] },
                            { "long_name": "M5V 2T6", "types": ["postal_code"] }
                        ]
                    }]
                }"#,
            );
    });

    let app = create_router(build_state(server.url("")));
    let response = app
        .oneshot(
            Request::builder()
                .uri("/api/search?query=M5V%202T6")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), 200);
    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let json: Value = serde_json::from_slice(&body).unwrap();

    assert_eq!(json["normalized_query"], "M5V 2T6");
    assert_eq!(json["results"][0]["postal_code"], "M5V 2T6");
}

#[tokio::test]
async fn search_invalid_query() {
    let app = create_router(build_state("https://example.test".to_string()));
    let response = app
        .oneshot(
            Request::builder()
                .uri("/api/search?query=%20%20")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), 400);
}
