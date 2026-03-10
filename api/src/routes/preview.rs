use std::sync::Arc;

use axum::routing::get;
use axum::Router;

use crate::handlers;
use crate::AppState;

pub fn preview_routes() -> Router<Arc<AppState>> {
    Router::new().route("/api/preview/poster", get(handlers::preview::preview_poster))
}
