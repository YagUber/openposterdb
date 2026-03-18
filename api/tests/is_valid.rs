mod common;

use axum::body::Body;
use axum::http::{Request, StatusCode};
use http_body_util::BodyExt;
use tower::ServiceExt;

fn json_body(json: serde_json::Value) -> Body {
    Body::from(json.to_string())
}

async fn set_free_api_key_enabled(app: &axum::Router, token: &str, enabled: bool) {
    let req = Request::builder()
        .method("PUT")
        .uri("/api/admin/settings")
        .header("content-type", "application/json")
        .header("authorization", format!("Bearer {token}"))
        .body(json_body(serde_json::json!({
            "poster_source": "t",
            "free_api_key_enabled": enabled
        })))
        .unwrap();
    let res = app.clone().oneshot(req).await.unwrap();
    assert_eq!(res.status(), StatusCode::OK);
}

#[tokio::test]
async fn is_valid_rejects_bogus_key() {
    let (app, _state) = common::setup_test_app().await;

    let req = Request::builder()
        .uri("/bogus-key/isValid")
        .body(Body::empty())
        .unwrap();

    let res = app.oneshot(req).await.unwrap();
    assert_eq!(res.status(), StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn is_valid_accepts_valid_key() {
    let (app, _state) = common::setup_test_app().await;
    let token = common::setup_admin(&app).await;

    // Create an API key
    let req = Request::builder()
        .method("POST")
        .uri("/api/keys")
        .header("content-type", "application/json")
        .header("authorization", format!("Bearer {token}"))
        .body(Body::from(r#"{"name":"valid-test"}"#))
        .unwrap();

    let res = app.clone().oneshot(req).await.unwrap();
    assert_eq!(res.status(), StatusCode::OK);
    let body = res.into_body().collect().await.unwrap().to_bytes();
    let json: serde_json::Value = serde_json::from_slice(&body).unwrap();
    let api_key = json["key"].as_str().unwrap();

    // Check isValid
    let req = Request::builder()
        .uri(format!("/{api_key}/isValid"))
        .body(Body::empty())
        .unwrap();

    let res = app.oneshot(req).await.unwrap();
    assert_eq!(res.status(), StatusCode::OK);
}

#[tokio::test]
async fn is_valid_free_key_accepted_when_enabled() {
    let (app, _state) = common::setup_test_app().await;
    let token = common::setup_admin(&app).await;

    set_free_api_key_enabled(&app, &token, true).await;

    let req = Request::builder()
        .uri("/t0-free-rpdb/isValid")
        .body(Body::empty())
        .unwrap();

    let res = app.oneshot(req).await.unwrap();
    assert_eq!(res.status(), StatusCode::OK);
}

#[tokio::test]
async fn is_valid_free_key_rejected_when_disabled() {
    let (app, _state) = common::setup_test_app().await;

    let req = Request::builder()
        .uri("/t0-free-rpdb/isValid")
        .body(Body::empty())
        .unwrap();

    let res = app.oneshot(req).await.unwrap();
    assert_eq!(res.status(), StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn is_valid_returns_json_body() {
    let (app, _state) = common::setup_test_app().await;
    let token = common::setup_admin(&app).await;

    let req = Request::builder()
        .method("POST")
        .uri("/api/keys")
        .header("content-type", "application/json")
        .header("authorization", format!("Bearer {token}"))
        .body(Body::from(r#"{"name":"body-test"}"#))
        .unwrap();

    let res = app.clone().oneshot(req).await.unwrap();
    let body = res.into_body().collect().await.unwrap().to_bytes();
    let json: serde_json::Value = serde_json::from_slice(&body).unwrap();
    let api_key = json["key"].as_str().unwrap();

    let req = Request::builder()
        .uri(format!("/{api_key}/isValid"))
        .body(Body::empty())
        .unwrap();

    let res = app.oneshot(req).await.unwrap();
    assert_eq!(res.status(), StatusCode::OK);
    let body = res.into_body().collect().await.unwrap().to_bytes();
    let json: serde_json::Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(json, serde_json::json!({ "valid": true }));
}

#[tokio::test]
async fn poster_with_valid_lang_param_accepted() {
    let (app, _state) = common::setup_test_app().await;
    let token = common::setup_admin(&app).await;

    let req = Request::builder()
        .method("POST")
        .uri("/api/keys")
        .header("content-type", "application/json")
        .header("authorization", format!("Bearer {token}"))
        .body(Body::from(r#"{"name":"lang-test"}"#))
        .unwrap();

    let res = app.clone().oneshot(req).await.unwrap();
    let body = res.into_body().collect().await.unwrap().to_bytes();
    let json: serde_json::Value = serde_json::from_slice(&body).unwrap();
    let api_key = json["key"].as_str().unwrap();

    // Request with ?lang=de — should not return 400 (lang is valid)
    let req = Request::builder()
        .uri(format!(
            "/{api_key}/imdb/poster-default/tt0000001.jpg?lang=de"
        ))
        .body(Body::empty())
        .unwrap();

    let res = app.oneshot(req).await.unwrap();
    // Should not be 400 (bad request) — the lang param is valid.
    // It may fail for other reasons (no TMDB key) but not due to lang validation.
    assert_ne!(res.status(), StatusCode::BAD_REQUEST);
    assert_ne!(res.status(), StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn poster_with_invalid_lang_param_rejected() {
    let (app, _state) = common::setup_test_app().await;
    let token = common::setup_admin(&app).await;

    let req = Request::builder()
        .method("POST")
        .uri("/api/keys")
        .header("content-type", "application/json")
        .header("authorization", format!("Bearer {token}"))
        .body(Body::from(r#"{"name":"lang-bad"}"#))
        .unwrap();

    let res = app.clone().oneshot(req).await.unwrap();
    let body = res.into_body().collect().await.unwrap().to_bytes();
    let json: serde_json::Value = serde_json::from_slice(&body).unwrap();
    let api_key = json["key"].as_str().unwrap();

    // Request with invalid lang (too long)
    let req = Request::builder()
        .uri(format!(
            "/{api_key}/imdb/poster-default/tt0000001.jpg?lang=toolongvalue"
        ))
        .body(Body::empty())
        .unwrap();

    let res = app.oneshot(req).await.unwrap();
    assert_eq!(res.status(), StatusCode::BAD_REQUEST);
}
