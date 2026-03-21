use std::sync::Arc;

use crate::error::AppError;
use crate::services::retry::{self, TMDB_API_RETRY, TMDB_CDN_RETRY};
use serde::de::DeserializeOwned;
use zeroize::Zeroizing;

#[derive(Clone)]
pub struct TmdbClient {
    api_key: Arc<Zeroizing<String>>,
    http: reqwest::Client,
}

impl TmdbClient {
    pub fn new(api_key: String, http: reqwest::Client) -> Self {
        Self { api_key: Arc::new(Zeroizing::new(api_key)), http }
    }

    pub async fn get<T: DeserializeOwned>(
        &self,
        path: &str,
        params: &[(&str, &str)],
    ) -> Result<T, AppError> {
        let url = format!("https://api.themoviedb.org/3{path}");
        let resp = retry::send_with_retry(&TMDB_API_RETRY, || {
            let mut req = self.http.get(&url).query(&[("api_key", self.api_key.as_str())]);
            if !params.is_empty() {
                req = req.query(params);
            }
            req.send()
        })
        .await?
        .error_for_status()?;
        Ok(resp.json().await?)
    }

    pub async fn fetch_poster_bytes(&self, poster_path: &str, tmdb_size: &str) -> Result<Vec<u8>, AppError> {
        self.fetch_image_bytes(poster_path, tmdb_size).await
    }

    /// Fetch poster bytes with If-Modified-Since. Returns `None` on 304 Not Modified.
    pub async fn fetch_poster_bytes_conditional(
        &self,
        poster_path: &str,
        tmdb_size: &str,
        if_modified_since: Option<std::time::SystemTime>,
    ) -> Result<Option<Vec<u8>>, AppError> {
        let url = format!("https://image.tmdb.org/t/p/{tmdb_size}{poster_path}");
        let since_header = if_modified_since.map(|t| {
            let dt: chrono::DateTime<chrono::Utc> = t.into();
            dt.format("%a, %d %b %Y %H:%M:%S GMT").to_string()
        });
        let resp = retry::send_with_retry(&TMDB_CDN_RETRY, || {
            let mut r = self.http.get(&url);
            if let Some(ref h) = since_header {
                r = r.header(reqwest::header::IF_MODIFIED_SINCE, h.as_str());
            }
            r.send()
        })
        .await?;
        if resp.status() == reqwest::StatusCode::NOT_MODIFIED {
            return Ok(None);
        }
        let resp = resp.error_for_status()?;
        Ok(Some(resp.bytes().await?.to_vec()))
    }

    pub async fn get_images(&self, media_type: &str, tmdb_id: u64, lang: &str) -> Result<TmdbImagesResponse, AppError> {
        let path = format!("/{media_type}/{tmdb_id}/images");
        let include_lang = if lang.is_empty() { "null".to_string() } else { format!("{lang},null") };
        self.get(&path, &[("include_image_language", &include_lang)]).await
    }

    /// Select the best image from a list of TMDB images.
    ///
    /// When `textless` is true, only null-language images (no text overlay) are
    /// considered — returns `None` if none exist so the caller can fall back to
    /// another source (e.g. fanart.tv).
    ///
    /// When `textless` is false and `lang` is non-empty: try requested lang,
    /// then English fallback. Returns `None` if neither matches so the caller
    /// uses the default image (e.g. `resolved.poster_path`) rather than
    /// silently returning a textless image.
    ///
    /// When `lang` is empty (e.g. backdrops): returns the best null-language
    /// image, since backdrops are inherently language-agnostic.
    pub fn select_image<'a>(images: &'a [TmdbImage], lang: &str, textless: bool) -> Option<&'a TmdbImage> {
        let find_best = |target: Option<&str>| -> Option<&TmdbImage> {
            images
                .iter()
                .filter(|img| img.iso_639_1.as_deref() == target)
                .max_by(|a, b| a.vote_average.partial_cmp(&b.vote_average).unwrap_or(std::cmp::Ordering::Equal))
        };

        if textless {
            return find_best(None);
        }
        // Language-agnostic request (e.g. backdrops) — return best null-language image
        if lang.is_empty() {
            return find_best(None);
        }
        // Try requested language, then English fallback
        if let Some(img) = find_best(Some(lang)) {
            return Some(img);
        }
        if lang != "en" {
            if let Some(img) = find_best(Some("en")) {
                return Some(img);
            }
        }
        // No match — return None so caller uses the default image
        None
    }

    /// Fetch image bytes from the TMDB CDN for any image type (poster, logo, backdrop).
    pub async fn fetch_image_bytes(&self, file_path: &str, size: &str) -> Result<Vec<u8>, AppError> {
        let url = format!("https://image.tmdb.org/t/p/{size}{file_path}");
        let resp = retry::send_with_retry(&TMDB_CDN_RETRY, || self.http.get(&url).send())
            .await?
            .error_for_status()?;
        Ok(resp.bytes().await?.to_vec())
    }
}

#[derive(Debug, Clone, serde::Deserialize)]
pub struct TmdbImage {
    pub file_path: String,
    pub iso_639_1: Option<String>,
    pub vote_average: f64,
}

#[derive(Debug, Clone, serde::Deserialize)]
pub struct TmdbImagesResponse {
    #[serde(default)]
    pub backdrops: Vec<TmdbImage>,
    #[serde(default)]
    pub logos: Vec<TmdbImage>,
    #[serde(default)]
    pub posters: Vec<TmdbImage>,
}

#[cfg(test)]
mod tests {
    use super::*;

    fn img(path: &str, lang: Option<&str>, vote: f64) -> TmdbImage {
        TmdbImage {
            file_path: path.to_string(),
            iso_639_1: lang.map(|s| s.to_string()),
            vote_average: vote,
        }
    }

    #[test]
    fn select_image_empty_list() {
        assert!(TmdbClient::select_image(&[], "en", false).is_none());
        assert!(TmdbClient::select_image(&[], "en", true).is_none());
    }

    #[test]
    fn select_image_exact_lang_match() {
        let images = vec![
            img("/en.jpg", Some("en"), 7.0),
            img("/de.jpg", Some("de"), 8.0),
        ];
        let selected = TmdbClient::select_image(&images, "de", false).unwrap();
        assert_eq!(selected.file_path, "/de.jpg");
    }

    #[test]
    fn select_image_falls_back_to_english() {
        let images = vec![
            img("/en.jpg", Some("en"), 6.0),
            img("/null.jpg", None, 9.0),
        ];
        // Requesting French, no French available → should fall back to English
        let selected = TmdbClient::select_image(&images, "fr", false).unwrap();
        assert_eq!(selected.file_path, "/en.jpg");
    }

    #[test]
    fn select_image_no_match_returns_none() {
        // Only textless (null-lang) images — should NOT be returned when textless=false
        let images = vec![
            img("/null.jpg", None, 9.0),
        ];
        assert!(TmdbClient::select_image(&images, "de", false).is_none());
    }

    #[test]
    fn select_image_english_request_no_english_returns_none() {
        // Requesting English, only null-lang available
        let images = vec![
            img("/null.jpg", None, 9.0),
        ];
        assert!(TmdbClient::select_image(&images, "en", false).is_none());
    }

    #[test]
    fn select_image_textless_returns_null_lang() {
        let images = vec![
            img("/en.jpg", Some("en"), 9.0),
            img("/null.jpg", None, 5.0),
        ];
        let selected = TmdbClient::select_image(&images, "en", true).unwrap();
        assert_eq!(selected.file_path, "/null.jpg");
    }

    #[test]
    fn select_image_textless_no_null_lang() {
        let images = vec![
            img("/en.jpg", Some("en"), 9.0),
        ];
        assert!(TmdbClient::select_image(&images, "en", true).is_none());
    }

    #[test]
    fn select_image_picks_highest_vote() {
        let images = vec![
            img("/en_low.jpg", Some("en"), 3.0),
            img("/en_high.jpg", Some("en"), 8.0),
            img("/en_mid.jpg", Some("en"), 5.0),
        ];
        let selected = TmdbClient::select_image(&images, "en", false).unwrap();
        assert_eq!(selected.file_path, "/en_high.jpg");
    }

    #[test]
    fn select_image_empty_lang_returns_null_lang() {
        // Backdrops use empty lang — should return null-language images
        let images = vec![
            img("/en.jpg", Some("en"), 9.0),
            img("/null.jpg", None, 5.0),
        ];
        let selected = TmdbClient::select_image(&images, "", false).unwrap();
        assert_eq!(selected.file_path, "/null.jpg");
    }

    #[test]
    fn select_image_empty_lang_no_null_returns_none() {
        let images = vec![
            img("/en.jpg", Some("en"), 9.0),
        ];
        assert!(TmdbClient::select_image(&images, "", false).is_none());
    }
}
