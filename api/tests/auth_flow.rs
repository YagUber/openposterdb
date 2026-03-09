mod common;

use axum::body::Body;
use axum::http::{Request, StatusCode};
use http_body_util::BodyExt;
use tower::ServiceExt;

fn json_body(json: serde_json::Value) -> Body {
    Body::from(json.to_string())
}

#[tokio::test]
async fn setup_creates_admin_and_returns_token() {
    let (app, _state) = common::setup_test_app().await;

    let req = Request::builder()
        .method("POST")
        .uri("/api/auth/setup")
        .header("content-type", "application/json")
        .body(json_body(serde_json::json!({
            "username": "admin",
            "password": "testpassword123"
        })))
        .unwrap();

    let res = app.oneshot(req).await.unwrap();
    assert_eq!(res.status(), StatusCode::OK);

    let set_cookie = common::extract_set_cookie(res.headers());
    assert!(set_cookie.is_some(), "should set refresh cookie");

    let body = res.into_body().collect().await.unwrap().to_bytes();
    let json: serde_json::Value = serde_json::from_slice(&body).unwrap();
    assert!(json["token"].is_string());
}

#[tokio::test]
async fn setup_fails_when_admin_exists() {
    let (app, _state) = common::setup_test_app().await;

    // First setup succeeds
    common::setup_admin(&app).await;

    // Second setup fails
    let req = Request::builder()
        .method("POST")
        .uri("/api/auth/setup")
        .header("content-type", "application/json")
        .body(json_body(serde_json::json!({
            "username": "admin2",
            "password": "testpassword123"
        })))
        .unwrap();

    let res = app.oneshot(req).await.unwrap();
    assert_eq!(res.status(), StatusCode::FORBIDDEN);
}

#[tokio::test]
async fn login_with_valid_credentials() {
    let (app, _state) = common::setup_test_app().await;
    common::setup_admin(&app).await;

    let req = Request::builder()
        .method("POST")
        .uri("/api/auth/login")
        .header("content-type", "application/json")
        .body(json_body(serde_json::json!({
            "username": "admin",
            "password": "testpassword123"
        })))
        .unwrap();

    let res = app.oneshot(req).await.unwrap();
    assert_eq!(res.status(), StatusCode::OK);

    let body = res.into_body().collect().await.unwrap().to_bytes();
    let json: serde_json::Value = serde_json::from_slice(&body).unwrap();
    assert!(json["token"].is_string());
}

#[tokio::test]
async fn login_with_wrong_password() {
    let (app, _state) = common::setup_test_app().await;
    common::setup_admin(&app).await;

    let req = Request::builder()
        .method("POST")
        .uri("/api/auth/login")
        .header("content-type", "application/json")
        .body(json_body(serde_json::json!({
            "username": "admin",
            "password": "wrongpassword"
        })))
        .unwrap();

    let res = app.oneshot(req).await.unwrap();
    assert_eq!(res.status(), StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn login_with_unknown_user() {
    let (app, _state) = common::setup_test_app().await;
    common::setup_admin(&app).await;

    let req = Request::builder()
        .method("POST")
        .uri("/api/auth/login")
        .header("content-type", "application/json")
        .body(json_body(serde_json::json!({
            "username": "nonexistent",
            "password": "testpassword123"
        })))
        .unwrap();

    let res = app.oneshot(req).await.unwrap();
    assert_eq!(res.status(), StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn refresh_with_valid_cookie() {
    let (app, _state) = common::setup_test_app().await;

    // Setup to get refresh token
    let req = Request::builder()
        .method("POST")
        .uri("/api/auth/setup")
        .header("content-type", "application/json")
        .body(json_body(serde_json::json!({
            "username": "admin",
            "password": "testpassword123"
        })))
        .unwrap();

    let res = app.clone().oneshot(req).await.unwrap();
    let set_cookie = common::extract_set_cookie(res.headers()).unwrap();
    let refresh_token = common::extract_refresh_token(&set_cookie).unwrap();

    // Use refresh token
    let req = Request::builder()
        .method("POST")
        .uri("/api/auth/refresh")
        .header("cookie", format!("refresh_token={refresh_token}"))
        .body(Body::empty())
        .unwrap();

    let res = app.oneshot(req).await.unwrap();
    assert_eq!(res.status(), StatusCode::OK);

    // Should get new tokens
    let new_set_cookie = common::extract_set_cookie(res.headers());
    assert!(new_set_cookie.is_some(), "should set new refresh cookie");

    let body = res.into_body().collect().await.unwrap().to_bytes();
    let json: serde_json::Value = serde_json::from_slice(&body).unwrap();
    assert!(json["token"].is_string());
}

#[tokio::test]
async fn refresh_token_rotation_invalidates_old() {
    let (app, _state) = common::setup_test_app().await;

    // Setup to get refresh token
    let req = Request::builder()
        .method("POST")
        .uri("/api/auth/setup")
        .header("content-type", "application/json")
        .body(json_body(serde_json::json!({
            "username": "admin",
            "password": "testpassword123"
        })))
        .unwrap();

    let res = app.clone().oneshot(req).await.unwrap();
    let set_cookie = common::extract_set_cookie(res.headers()).unwrap();
    let old_refresh = common::extract_refresh_token(&set_cookie).unwrap();

    // Use refresh token (first time — should work)
    let req = Request::builder()
        .method("POST")
        .uri("/api/auth/refresh")
        .header("cookie", format!("refresh_token={old_refresh}"))
        .body(Body::empty())
        .unwrap();

    let res = app.clone().oneshot(req).await.unwrap();
    assert_eq!(res.status(), StatusCode::OK);

    // Use same old refresh token again — should fail (rotated)
    let req = Request::builder()
        .method("POST")
        .uri("/api/auth/refresh")
        .header("cookie", format!("refresh_token={old_refresh}"))
        .body(Body::empty())
        .unwrap();

    let res = app.oneshot(req).await.unwrap();
    assert_eq!(res.status(), StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn refresh_without_cookie_returns_401() {
    let (app, _state) = common::setup_test_app().await;

    let req = Request::builder()
        .method("POST")
        .uri("/api/auth/refresh")
        .body(Body::empty())
        .unwrap();

    let res = app.oneshot(req).await.unwrap();
    assert_eq!(res.status(), StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn logout_clears_tokens() {
    let (app, _state) = common::setup_test_app().await;
    let token = common::setup_admin(&app).await;

    let req = Request::builder()
        .method("POST")
        .uri("/api/auth/logout")
        .header("authorization", format!("Bearer {token}"))
        .body(Body::empty())
        .unwrap();

    let res = app.oneshot(req).await.unwrap();
    assert_eq!(res.status(), StatusCode::OK);

    let set_cookie = common::extract_set_cookie(res.headers()).unwrap();
    assert!(set_cookie.contains("Max-Age=0"), "should clear cookie");
}

#[tokio::test]
async fn auth_status_setup_required_true() {
    let (app, _state) = common::setup_test_app().await;

    let req = Request::builder()
        .uri("/api/auth/status")
        .body(Body::empty())
        .unwrap();

    let res = app.oneshot(req).await.unwrap();
    assert_eq!(res.status(), StatusCode::OK);

    let body = res.into_body().collect().await.unwrap().to_bytes();
    let json: serde_json::Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(json["setup_required"], true);
}

#[tokio::test]
async fn auth_status_setup_required_false() {
    let (app, _state) = common::setup_test_app().await;
    common::setup_admin(&app).await;

    let req = Request::builder()
        .uri("/api/auth/status")
        .body(Body::empty())
        .unwrap();

    let res = app.oneshot(req).await.unwrap();
    assert_eq!(res.status(), StatusCode::OK);

    let body = res.into_body().collect().await.unwrap().to_bytes();
    let json: serde_json::Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(json["setup_required"], false);
}
