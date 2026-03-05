use axum::{body::Body, http::Request};
use httpmock::MockServer;
use serde_json::Value;
use tower::util::ServiceExt;

use backend::api::{create_router, AppState};
use backend::db::Database;

fn build_state(base_url: String) -> AppState {
    let db = Database::new_in_memory().expect("db init");
    db.initialize_schema().expect("schema init");

    AppState {
        db: std::sync::Arc::new(db),
        geocode_client: reqwest::Client::new(),
        geocode_cache: std::sync::Arc::new(std::sync::Mutex::new(std::collections::HashMap::new())),
        geocode_base_url: base_url,
    }
}

#[tokio::test]
async fn search_city_success() {
    let server = MockServer::start();
    let _mock = server.mock(|when, then| {
        when.method("GET")
            .path("/search")
            .query_param("format", "json")
            .query_param("addressdetails", "1")
            .query_param("limit", "1")
            .query_param("countrycodes", "ca");
        then.status(200)
            .header("content-type", "application/json")
            .body(
                r#"[{
                    "lat": "43.653226",
                    "lon": "-79.3831843",
                    "address": {
                        "city": "Toronto",
                        "state": "Ontario",
                        "postcode": "M5V 2T6"
                    }
                }]"#,
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
}

#[tokio::test]
async fn search_postal_success() {
    let server = MockServer::start();
    let _mock = server.mock(|when, then| {
        when.method("GET")
            .path("/search")
            .query_param("format", "json")
            .query_param("addressdetails", "1")
            .query_param("limit", "1")
            .query_param("countrycodes", "ca");
        then.status(200)
            .header("content-type", "application/json")
            .body(
                r#"[{
                    "lat": "43.651070",
                    "lon": "-79.347015",
                    "address": {
                        "city": "Toronto",
                        "state": "Ontario",
                        "postcode": "M5V 2T6"
                    }
                }]"#,
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
