mod common;

use axum::body::Body;
use axum::http::{Request, StatusCode};
use http_body_util::BodyExt;
use tower::ServiceExt;

/// Helper: create an admin and API key, return the raw key.
async fn create_api_key(app: &axum::Router) -> String {
    let token = common::setup_admin(app).await;

    let req = Request::builder()
        .method("POST")
        .uri("/api/keys")
        .header("content-type", "application/json")
        .header("authorization", format!("Bearer {token}"))
        .body(Body::from(r#"{"name":"override-test"}"#))
        .unwrap();

    let res = app.clone().oneshot(req).await.unwrap();
    assert_eq!(res.status(), StatusCode::OK);
    let body = res.into_body().collect().await.unwrap().to_bytes();
    let json: serde_json::Value = serde_json::from_slice(&body).unwrap();
    json["key"].as_str().unwrap().to_string()
}

// --- Valid override parameters accepted ---

#[tokio::test]
async fn poster_badge_style_accepted() {
    let (app, _) = common::setup_test_app().await;
    let api_key = create_api_key(&app).await;

    let req = Request::builder()
        .uri(format!(
            "/{api_key}/imdb/poster-default/tt0000001.jpg?badge_style=h"
        ))
        .body(Body::empty())
        .unwrap();

    let res = app.oneshot(req).await.unwrap();
    assert_ne!(res.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn poster_ratings_limit_accepted() {
    let (app, _) = common::setup_test_app().await;
    let api_key = create_api_key(&app).await;

    let req = Request::builder()
        .uri(format!(
            "/{api_key}/imdb/poster-default/tt0000001.jpg?ratings_limit=3"
        ))
        .body(Body::empty())
        .unwrap();

    let res = app.oneshot(req).await.unwrap();
    assert_ne!(res.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn poster_all_overrides_accepted() {
    let (app, _) = common::setup_test_app().await;
    let api_key = create_api_key(&app).await;

    let req = Request::builder()
        .uri(format!(
            "/{api_key}/imdb/poster-default/tt0000001.jpg?badge_style=v&label_style=i&badge_size=l&badge_direction=h&position=tl&ratings_limit=5&ratings_order=imdb,tmdb&poster_source=t&fanart_textless=false"
        ))
        .body(Body::empty())
        .unwrap();

    let res = app.oneshot(req).await.unwrap();
    assert_ne!(res.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn logo_badge_style_accepted() {
    let (app, _) = common::setup_test_app().await;
    let api_key = create_api_key(&app).await;

    let req = Request::builder()
        .uri(format!(
            "/{api_key}/imdb/logo-default/tt0000001.png?badge_style=h&ratings_limit=2&label_style=o"
        ))
        .body(Body::empty())
        .unwrap();

    let res = app.oneshot(req).await.unwrap();
    assert_ne!(res.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn backdrop_badge_size_accepted() {
    let (app, _) = common::setup_test_app().await;
    let api_key = create_api_key(&app).await;

    let req = Request::builder()
        .uri(format!(
            "/{api_key}/imdb/backdrop-default/tt0000001.jpg?badge_size=xl&badge_style=v"
        ))
        .body(Body::empty())
        .unwrap();

    let res = app.oneshot(req).await.unwrap();
    assert_ne!(res.status(), StatusCode::BAD_REQUEST);
}

// --- Invalid override parameters rejected ---

#[tokio::test]
async fn poster_invalid_ratings_limit_rejected() {
    let (app, _) = common::setup_test_app().await;
    let api_key = create_api_key(&app).await;

    let req = Request::builder()
        .uri(format!(
            "/{api_key}/imdb/poster-default/tt0000001.jpg?ratings_limit=9"
        ))
        .body(Body::empty())
        .unwrap();

    let res = app.oneshot(req).await.unwrap();
    assert_eq!(res.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn poster_invalid_ratings_order_rejected() {
    let (app, _) = common::setup_test_app().await;
    let api_key = create_api_key(&app).await;

    let req = Request::builder()
        .uri(format!(
            "/{api_key}/imdb/poster-default/tt0000001.jpg?ratings_order=bogus_source"
        ))
        .body(Body::empty())
        .unwrap();

    let res = app.oneshot(req).await.unwrap();
    assert_eq!(res.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn poster_invalid_badge_style_rejected() {
    let (app, _) = common::setup_test_app().await;
    let api_key = create_api_key(&app).await;

    let req = Request::builder()
        .uri(format!(
            "/{api_key}/imdb/poster-default/tt0000001.jpg?badge_style=z"
        ))
        .body(Body::empty())
        .unwrap();

    let res = app.oneshot(req).await.unwrap();
    assert_eq!(res.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn poster_negative_ratings_limit_rejected() {
    let (app, _) = common::setup_test_app().await;
    let api_key = create_api_key(&app).await;

    let req = Request::builder()
        .uri(format!(
            "/{api_key}/imdb/poster-default/tt0000001.jpg?ratings_limit=-1"
        ))
        .body(Body::empty())
        .unwrap();

    let res = app.oneshot(req).await.unwrap();
    assert_eq!(res.status(), StatusCode::BAD_REQUEST);
}

// --- Poster-only params silently ignored on logo/backdrop ---

#[tokio::test]
async fn logo_poster_only_params_ignored() {
    let (app, _) = common::setup_test_app().await;
    let api_key = create_api_key(&app).await;

    // position and badge_direction are poster-only but should not cause errors on logo
    let req = Request::builder()
        .uri(format!(
            "/{api_key}/imdb/logo-default/tt0000001.png?position=tl&badge_direction=h&poster_source=f&fanart_textless=true"
        ))
        .body(Body::empty())
        .unwrap();

    let res = app.oneshot(req).await.unwrap();
    assert_ne!(res.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn backdrop_poster_only_params_ignored() {
    let (app, _) = common::setup_test_app().await;
    let api_key = create_api_key(&app).await;

    let req = Request::builder()
        .uri(format!(
            "/{api_key}/imdb/backdrop-default/tt0000001.jpg?position=br&badge_direction=v&poster_source=t"
        ))
        .body(Body::empty())
        .unwrap();

    let res = app.oneshot(req).await.unwrap();
    assert_ne!(res.status(), StatusCode::BAD_REQUEST);
}

// --- No overrides still works ---

#[tokio::test]
async fn poster_no_overrides_accepted() {
    let (app, _) = common::setup_test_app().await;
    let api_key = create_api_key(&app).await;

    let req = Request::builder()
        .uri(format!(
            "/{api_key}/imdb/poster-default/tt0000001.jpg"
        ))
        .body(Body::empty())
        .unwrap();

    let res = app.oneshot(req).await.unwrap();
    assert_ne!(res.status(), StatusCode::BAD_REQUEST);
}

// --- Ratings limit boundary values ---

#[tokio::test]
async fn poster_ratings_limit_zero_accepted() {
    let (app, _) = common::setup_test_app().await;
    let api_key = create_api_key(&app).await;

    let req = Request::builder()
        .uri(format!(
            "/{api_key}/imdb/poster-default/tt0000001.jpg?ratings_limit=0"
        ))
        .body(Body::empty())
        .unwrap();

    let res = app.oneshot(req).await.unwrap();
    assert_ne!(res.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn poster_ratings_limit_eight_accepted() {
    let (app, _) = common::setup_test_app().await;
    let api_key = create_api_key(&app).await;

    let req = Request::builder()
        .uri(format!(
            "/{api_key}/imdb/poster-default/tt0000001.jpg?ratings_limit=8"
        ))
        .body(Body::empty())
        .unwrap();

    let res = app.oneshot(req).await.unwrap();
    assert_ne!(res.status(), StatusCode::BAD_REQUEST);
}
