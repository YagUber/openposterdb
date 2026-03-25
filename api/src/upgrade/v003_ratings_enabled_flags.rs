use sea_orm::{ConnectionTrait, DatabaseConnection};

use crate::error::AppError;

/// Migrate existing rows that used `ratings_limit = 0` as a "disabled" sentinel.
///
/// Before this change, a limit of 0 meant ratings were disabled. Now a separate
/// `ratings_enabled` boolean column carries that state, and the limit always
/// holds a meaningful display value (≥ 1). This upgrade:
///
/// 1. Sets `*_ratings_enabled = 0` for any row whose limit was 0.
/// 2. Resets those limits to their default values so the UI shows a sensible number.
pub async fn run(
    db: &DatabaseConnection,
    _cache_dir: &str,
    _external_cache_only: bool,
) -> Result<(), AppError> {
    let mut total = 0u64;

    let result = db
        .execute(sea_orm::Statement::from_string(
            sea_orm::DatabaseBackend::Sqlite,
            "UPDATE api_key_settings SET ratings_enabled = 0, ratings_limit = 3 WHERE ratings_limit = 0",
        ))
        .await?;
    total += result.rows_affected();

    let result = db
        .execute(sea_orm::Statement::from_string(
            sea_orm::DatabaseBackend::Sqlite,
            "UPDATE api_key_settings SET logo_ratings_enabled = 0, logo_ratings_limit = 5 WHERE logo_ratings_limit = 0",
        ))
        .await?;
    total += result.rows_affected();

    let result = db
        .execute(sea_orm::Statement::from_string(
            sea_orm::DatabaseBackend::Sqlite,
            "UPDATE api_key_settings SET backdrop_ratings_enabled = 0, backdrop_ratings_limit = 5 WHERE backdrop_ratings_limit = 0",
        ))
        .await?;
    total += result.rows_affected();

    let result = db
        .execute(sea_orm::Statement::from_string(
            sea_orm::DatabaseBackend::Sqlite,
            "UPDATE api_key_settings SET episode_ratings_enabled = 0, episode_ratings_limit = 1 WHERE episode_ratings_limit = 0",
        ))
        .await?;
    total += result.rows_affected();

    tracing::info!(rows = total, "migrated ratings_limit=0 sentinel rows to ratings_enabled=false");
    Ok(())
}
