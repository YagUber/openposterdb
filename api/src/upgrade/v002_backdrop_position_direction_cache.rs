use sea_orm::{ConnectionTrait, DatabaseConnection};

use crate::error::AppError;

/// Migrate backdrop cache keys to include position and direction suffixes.
///
/// Before this change, backdrop cache suffixes were:
///   `{ratings}.s{style}.l{label}.b{badge_size}.z{image_size}`
/// After this change they include position (`.ptr`) and direction (`.dv`):
///   `{ratings}.ptr.s{style}.l{label}.dv.b{badge_size}.z{image_size}`
///
/// All existing backdrops were rendered with the hardcoded defaults
/// (TopRight position, Vertical direction), so we insert those defaults.
pub async fn run(
    db: &DatabaseConnection,
    cache_dir: &str,
    external_cache_only: bool,
) -> Result<(), AppError> {
    run_db(db).await?;
    run_fs(cache_dir, external_cache_only).await?;
    Ok(())
}

/// Database step — insert `.ptr` before badge style and `.dv` after label style
/// in all backdrop cache keys.
pub async fn run_db(db: &impl ConnectionTrait) -> Result<(), AppError> {
    let mut total = 0u64;

    // Insert `.ptr` (default position = TopRight) before badge style (`.sv`, `.sh`, `.sd`)
    // Use substr-based replacement to only replace the first occurrence of the pattern.
    for style in ["sv", "sh", "sd"] {
        let old = format!(".{style}.l");
        let new = format!(".ptr.{style}.l");
        let old_len = old.len() as i32;
        let result = db
            .execute(sea_orm::Statement::from_string(
                sea_orm::DatabaseBackend::Sqlite,
                format!(
                    "UPDATE image_meta SET cache_key = \
                     substr(cache_key, 1, instr(cache_key, '{old}') - 1) || '{new}' || \
                     substr(cache_key, instr(cache_key, '{old}') + {old_len}) \
                     WHERE image_type = 'b' AND instr(cache_key, '{old}') > 0 AND instr(cache_key, '.ptr.') = 0"
                ),
            ))
            .await?;
        total += result.rows_affected();
    }

    // Insert `.dv` (default direction = Vertical) after label style, before badge size
    for label in ["lt", "li", "lo"] {
        let old = format!(".{label}.b");
        let new = format!(".{label}.dv.b");
        let old_len = old.len() as i32;
        let result = db
            .execute(sea_orm::Statement::from_string(
                sea_orm::DatabaseBackend::Sqlite,
                format!(
                    "UPDATE image_meta SET cache_key = \
                     substr(cache_key, 1, instr(cache_key, '{old}') - 1) || '{new}' || \
                     substr(cache_key, instr(cache_key, '{old}') + {old_len}) \
                     WHERE image_type = 'b' AND instr(cache_key, '{old}') > 0 AND instr(cache_key, '.dv.') = 0"
                ),
            ))
            .await?;
        total += result.rows_affected();
    }

    tracing::info!(db_rows = total, "backdrop cache keys migrated (added position/direction suffixes)");
    Ok(())
}

/// Filesystem step — rename backdrop cache files to include position/direction suffixes.
pub async fn run_fs(cache_dir: &str, external_cache_only: bool) -> Result<(), AppError> {
    if external_cache_only {
        tracing::info!("backdrop position/direction filesystem rename skipped (external_cache_only)");
        return Ok(());
    }

    let backdrop_dir = std::path::Path::new(cache_dir).join("backdrops");
    let renamed = tokio::task::spawn_blocking(move || rename_files(&backdrop_dir))
        .await
        .map_err(|e| AppError::Other(format!("rename task panicked: {e}")))?
        ?;
    tracing::info!(fs_renamed = renamed, "backdrop cache files renamed (added position/direction suffixes)");
    Ok(())
}

/// Recursively rename backdrop cache files to include `.ptr` and `.dv` suffixes.
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
            if let Some(new_name) = migrate_name(name) {
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

/// Transform an old backdrop filename to include default position/direction suffixes.
///
/// Returns `None` if already migrated or no matching pattern found.
fn migrate_name(name: &str) -> Option<String> {
    // Already migrated — has both position and direction suffixes
    if name.contains(".ptr.") && name.contains(".dv.") {
        return None;
    }

    let mut result = name.to_string();
    let mut modified = false;

    // Insert `.ptr` before badge style (`.sv.l`, `.sh.l`, or `.sd.l`)
    if !result.contains(".ptr.") {
        for style in [".sv.l", ".sh.l", ".sd.l"] {
            if let Some(pos) = result.find(style) {
                result.insert_str(pos, ".ptr");
                modified = true;
                break;
            }
        }
    }

    // Insert `.dv` after label style, before badge size
    if !result.contains(".dv.") {
        for (old, new) in [(".lt.b", ".lt.dv.b"), (".li.b", ".li.dv.b"), (".lo.b", ".lo.dv.b")] {
            if result.contains(old) {
                result = result.replacen(old, new, 1);
                modified = true;
                break;
            }
        }
    }

    if modified { Some(result) } else { None }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn migrate_name_typical() {
        assert_eq!(
            migrate_name("tt1234567_b_f@mil.sv.lt.bm.zm.jpg"),
            Some("tt1234567_b_f@mil.ptr.sv.lt.dv.bm.zm.jpg".to_string()),
        );
    }

    #[test]
    fn migrate_name_horizontal_style_icon_label() {
        assert_eq!(
            migrate_name("tt1234567_b_t@ir.sh.li.bl.zl.jpg"),
            Some("tt1234567_b_t@ir.ptr.sh.li.dv.bl.zl.jpg".to_string()),
        );
    }

    #[test]
    fn migrate_name_default_style_official_label() {
        assert_eq!(
            migrate_name("tt1234567_b_f@m.sd.lo.bxs.zs.jpg"),
            Some("tt1234567_b_f@m.ptr.sd.lo.dv.bxs.zs.jpg".to_string()),
        );
    }

    #[test]
    fn migrate_name_already_migrated() {
        assert_eq!(
            migrate_name("tt1234567_b_f@mil.ptr.sv.lt.dv.bm.zm.jpg"),
            None,
        );
    }

    #[test]
    fn migrate_name_no_match() {
        // Not a backdrop cache file
        assert_eq!(migrate_name("tt1234567.jpg"), None);
    }

    #[test]
    fn migrate_name_extra_small_badge_size() {
        assert_eq!(
            migrate_name("tt1234567_b_f@mil.sv.lt.bxs.zm.jpg"),
            Some("tt1234567_b_f@mil.ptr.sv.lt.dv.bxs.zm.jpg".to_string()),
        );
    }

    #[test]
    fn migrate_name_extra_large_badge_size() {
        assert_eq!(
            migrate_name("tt1234567_b_f@mil.sv.lt.bxl.zvl.jpg"),
            Some("tt1234567_b_f@mil.ptr.sv.lt.dv.bxl.zvl.jpg".to_string()),
        );
    }

    #[test]
    fn migrate_name_zero_ratings() {
        assert_eq!(
            migrate_name("tt1234567_b_f@.sv.lt.bm.zm.jpg"),
            Some("tt1234567_b_f@.ptr.sv.lt.dv.bm.zm.jpg".to_string()),
        );
    }
}
