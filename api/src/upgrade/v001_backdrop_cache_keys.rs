use sea_orm::{ConnectionTrait, DatabaseConnection};

use crate::error::AppError;

/// Migrate backdrop cache keys from `_b@` to `_b_f@`.
///
/// Before this change, logos and backdrops were served exclusively via fanart.tv
/// and backdrop cache keys used the kind prefix `_b` with no source marker.
/// The new multi-source architecture adds `_t` (TMDB) and `_f` (Fanart.tv)
/// source markers. Existing backdrops (all from fanart.tv) need `_f` appended.
pub async fn run(
    db: &DatabaseConnection,
    cache_dir: &str,
    external_cache_only: bool,
) -> Result<(), AppError> {
    run_db(db).await?;
    run_fs(cache_dir, external_cache_only).await?;
    Ok(())
}

/// Database step — idempotent.
pub async fn run_db(db: &impl ConnectionTrait) -> Result<(), AppError> {
    let result = db
        .execute(sea_orm::Statement::from_string(
            sea_orm::DatabaseBackend::Sqlite,
            "UPDATE image_meta SET cache_key = replace(cache_key, '_b@', '_b_f@') \
             WHERE image_type = 'b' AND instr(cache_key, '_b@') > 0"
                .to_string(),
        ))
        .await?;
    tracing::info!(db_rows = result.rows_affected(), "backdrop cache keys migrated in DB (_b@ → _b_f@)");
    Ok(())
}

/// Migrate backdrop cache keys — filesystem step (idempotent).
///
/// Renames files containing `_b@` to `_b_f@`. Safe to call multiple times;
/// already-renamed files are skipped.
pub async fn run_fs(cache_dir: &str, external_cache_only: bool) -> Result<(), AppError> {
    if external_cache_only {
        tracing::info!("backdrop filesystem rename skipped (external_cache_only)");
        return Ok(());
    }

    let backdrop_dir = std::path::Path::new(cache_dir).join("backdrops");
    let renamed = tokio::task::spawn_blocking(move || rename_files(&backdrop_dir))
        .await
        .map_err(|e| AppError::Other(format!("rename task panicked: {e}")))?
        ?;
    tracing::info!(fs_renamed = renamed, "backdrop cache files renamed (_b@ → _b_f@)");
    Ok(())
}

/// Recursively rename files containing `_b@` to `_b_f@` in the given directory.
fn rename_files(dir: &std::path::Path) -> Result<u64, AppError> {
    let entries = match std::fs::read_dir(dir) {
        Ok(e) => e,
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Ok(0),
        Err(e) => {
            return Err(AppError::Other(format!(
                "failed to read {}: {e}",
                dir.display()
            )))
        }
    };

    let mut count = 0u64;
    for entry in entries {
        let entry = entry.map_err(|e| AppError::Other(e.to_string()))?;
        let path = entry.path();
        if path.is_dir() {
            count += rename_files(&path)?;
        } else if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
            if name.contains("_b@") && !name.contains("_b_f@") && !name.contains("_b_t@") {
                let new_name = name.replace("_b@", "_b_f@");
                let new_path = path.with_file_name(new_name);
                std::fs::rename(&path, &new_path).map_err(|e| {
                    AppError::Other(format!(
                        "rename failed: {} → {}: {e}",
                        path.display(),
                        new_path.display()
                    ))
                })?;
                count += 1;
            }
        }
    }

    Ok(count)
}
