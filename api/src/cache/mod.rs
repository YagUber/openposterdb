use std::path::{Path, PathBuf};
use std::time::SystemTime;
use tokio::fs;

use crate::error::AppError;

pub struct CacheEntry {
    pub bytes: Vec<u8>,
    pub is_stale: bool,
}

pub fn cache_path(cache_dir: &str, id_type: &str, id_value: &str) -> PathBuf {
    Path::new(cache_dir).join(id_type).join(format!("{id_value}.jpg"))
}

pub fn poster_cache_path(cache_dir: &str, poster_path: &str) -> PathBuf {
    // poster_path is like "/abc123.jpg" from TMDB
    let filename = poster_path.trim_start_matches('/');
    Path::new(cache_dir).join("posters").join(filename)
}

/// Read a cached file. `stale_secs = 0` means never stale.
pub async fn read(path: &Path, stale_secs: u64) -> Option<CacheEntry> {
    let bytes = fs::read(path).await.ok()?;
    let metadata = fs::metadata(path).await.ok()?;
    let modified = metadata.modified().ok()?;
    let age = SystemTime::now()
        .duration_since(modified)
        .unwrap_or_default()
        .as_secs();

    Some(CacheEntry {
        bytes,
        is_stale: stale_secs > 0 && age > stale_secs,
    })
}

pub async fn write(path: &Path, bytes: &[u8]) -> Result<(), AppError> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).await?;
    }
    fs::write(path, bytes).await?;
    Ok(())
}
