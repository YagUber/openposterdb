use std::sync::Arc;

use axum::Router;
#[cfg(not(any(test, feature = "test-support")))]
use axum::http::Request;

use crate::handlers::image;
use crate::AppState;

/// Extract client IP from `X-Forwarded-For` or `X-Real-IP` headers.
#[cfg(not(any(test, feature = "test-support")))]
fn extract_client_ip<T>(req: &Request<T>) -> String {
    req.headers()
        .get("x-forwarded-for")
        .and_then(|v| v.to_str().ok())
        .and_then(|s| s.split(',').next())
        .map(|s| s.trim().to_string())
        .or_else(|| {
            req.headers()
                .get("x-real-ip")
                .and_then(|v| v.to_str().ok())
                .map(|s| s.to_string())
        })
        .unwrap_or_else(|| "unknown".to_string())
}

#[cfg(not(any(test, feature = "test-support")))]
#[derive(Debug, Clone)]
struct ImageKeyExtractor;

#[cfg(not(any(test, feature = "test-support")))]
impl tower_governor::key_extractor::KeyExtractor for ImageKeyExtractor {
    type Key = String;

    fn extract<T>(
        &self,
        req: &Request<T>,
    ) -> Result<Self::Key, tower_governor::GovernorError> {
        let path = req.uri().path();
        let api_key = path.split('/').nth(1).unwrap_or("unknown");
        let key_prefix = &api_key[..api_key.len().min(16)];
        let ip = extract_client_ip(req);
        Ok(format!("{key_prefix}:{ip}"))
    }
}

/// Rate-limit key extractor for unauthenticated `/c/` CDN routes — uses IP only.
#[cfg(not(any(test, feature = "test-support")))]
#[derive(Debug, Clone)]
struct IpKeyExtractor;

#[cfg(not(any(test, feature = "test-support")))]
impl tower_governor::key_extractor::KeyExtractor for IpKeyExtractor {
    type Key = String;

    fn extract<T>(
        &self,
        req: &Request<T>,
    ) -> Result<Self::Key, tower_governor::GovernorError> {
        Ok(extract_client_ip(req))
    }
}

/// Poster, logo, and backdrop routes with `ImageKeyExtractor` rate limiting.
pub fn image_routes() -> Router<Arc<AppState>> {
    let router = Router::new()
        .route(
            "/{api_key}/{id_type}/poster-default/{id_value}",
            axum::routing::get(image::handler),
        )
        .route(
            "/{api_key}/{id_type}/logo-default/{id_value}",
            axum::routing::get(image::logo_handler),
        )
        .route(
            "/{api_key}/{id_type}/backdrop-default/{id_value}",
            axum::routing::get(image::backdrop_handler),
        );

    #[cfg(not(any(test, feature = "test-support")))]
    let router = {
        use tower_governor::{governor::GovernorConfigBuilder, GovernorLayer};

        let governor_conf = GovernorConfigBuilder::default()
            .per_millisecond(200)
            .burst_size(240)
            .key_extractor(ImageKeyExtractor)
            .finish()
            .expect("valid governor config");

        router.layer(GovernorLayer::new(governor_conf))
    };

    router
}

/// isValid route with lighter rate limiting.
pub fn is_valid_route() -> Router<Arc<AppState>> {
    let router = Router::new().route(
        "/{api_key}/isValid",
        axum::routing::get(image::is_valid_handler),
    );

    #[cfg(not(any(test, feature = "test-support")))]
    let router = {
        use tower_governor::{governor::GovernorConfigBuilder, GovernorLayer};

        let governor_conf = GovernorConfigBuilder::default()
            .per_millisecond(100) // 10 req/s
            .burst_size(30)
            .key_extractor(ImageKeyExtractor)
            .finish()
            .expect("valid governor config");

        router.layer(GovernorLayer::new(governor_conf))
    };

    router
}

/// CDN routes with `IpKeyExtractor` rate limiting.
pub fn cdn_routes() -> Router<Arc<AppState>> {
    let router = Router::new()
        .route(
            "/c/{settings_hash}/{id_type}/poster-default/{id_value}",
            axum::routing::get(image::cdn_poster_handler),
        )
        .route(
            "/c/{settings_hash}/{id_type}/logo-default/{id_value}",
            axum::routing::get(image::cdn_logo_handler),
        )
        .route(
            "/c/{settings_hash}/{id_type}/backdrop-default/{id_value}",
            axum::routing::get(image::cdn_backdrop_handler),
        );

    #[cfg(not(any(test, feature = "test-support")))]
    let router = {
        use tower_governor::{governor::GovernorConfigBuilder, GovernorLayer};

        let governor_conf = GovernorConfigBuilder::default()
            .per_millisecond(200)
            .burst_size(240)
            .key_extractor(IpKeyExtractor)
            .finish()
            .expect("valid governor config");

        router.layer(GovernorLayer::new(governor_conf))
    };

    router
}
