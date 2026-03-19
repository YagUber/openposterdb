use axum::extract::{Query, State};
use axum::http::header;
use axum::response::{IntoResponse, Response};
use image::codecs::png::PngEncoder;
use image::{ImageEncoder, Rgba, RgbaImage};
use serde::Deserialize;
use std::sync::{Arc, LazyLock};

use crate::cache;
use crate::error::AppError;
use crate::image::generate;
use crate::image::serve;
use crate::services::db::{self, BadgeDirection, BadgeSize, BadgeStyle, LabelStyle, PosterPosition};
use crate::services::ratings::{self, RatingBadge, RatingSource};
use crate::AppState;

/// A 500x750 dark gray gradient poster, computed once.
static SAMPLE_POSTER_PNG: LazyLock<Vec<u8>> = LazyLock::new(|| {
    let width = 500u32;
    let height = 750u32;
    let img = RgbaImage::from_fn(width, height, |_x, y| {
        // Dark gradient from #2a2a2a at top to #1a1a1a at bottom
        let t = y as f32 / height as f32;
        let v = (42.0 - t * 16.0) as u8;
        Rgba([v, v, v, 255])
    });
    let mut buf = Vec::new();
    let encoder = PngEncoder::new(&mut buf);
    encoder
        .write_image(img.as_raw(), width, height, image::ExtendedColorType::Rgba8)
        .expect("PNG encoding should not fail");
    buf
});

/// A 500x200 sample logo (white text-like shape on transparent background).
static SAMPLE_LOGO_PNG: LazyLock<Vec<u8>> = LazyLock::new(|| {
    let width = 400u32;
    let height = 120u32;
    let img = RgbaImage::from_fn(width, height, |x, y| {
        // Simple rounded rectangle shape to simulate a logo
        let margin = 8u32;
        if x >= margin && x < width - margin && y >= margin && y < height - margin {
            Rgba([220, 220, 220, 240])
        } else {
            Rgba([0, 0, 0, 0])
        }
    });
    let mut buf = Vec::new();
    let encoder = PngEncoder::new(&mut buf);
    encoder
        .write_image(img.as_raw(), width, height, image::ExtendedColorType::Rgba8)
        .expect("PNG encoding should not fail");
    buf
});

/// A 1280x720 dark gradient backdrop, computed once.
static SAMPLE_BACKDROP_PNG: LazyLock<Vec<u8>> = LazyLock::new(|| {
    let width = 1280u32;
    let height = 720u32;
    let img = RgbaImage::from_fn(width, height, |x, _y| {
        // Horizontal gradient from #1a1a2a (left) to #2a1a1a (right)
        let t = x as f32 / width as f32;
        let r = (26.0 + t * 16.0) as u8;
        let b = (42.0 - t * 16.0) as u8;
        Rgba([r, 26, b, 255])
    });
    let mut buf = Vec::new();
    let encoder = PngEncoder::new(&mut buf);
    encoder
        .write_image(img.as_raw(), width, height, image::ExtendedColorType::Rgba8)
        .expect("PNG encoding should not fail");
    buf
});

fn sample_badges() -> Vec<RatingBadge> {
    vec![
        RatingBadge { source: RatingSource::Imdb, value: "10.0".into() },
        RatingBadge { source: RatingSource::Tmdb, value: "100%".into() },
        RatingBadge { source: RatingSource::Rt, value: "100%".into() },
        RatingBadge { source: RatingSource::RtAudience, value: "100%".into() },
        RatingBadge { source: RatingSource::Metacritic, value: "100".into() },
        RatingBadge { source: RatingSource::Trakt, value: "100%".into() },
        RatingBadge { source: RatingSource::Letterboxd, value: "5.0".into() },
        RatingBadge { source: RatingSource::Mal, value: "10.00".into() },
    ]
}

#[derive(Debug, Deserialize)]
pub struct PreviewQuery {
    #[serde(default = "default_ratings_limit")]
    pub ratings_limit: i32,
    #[serde(default)]
    pub ratings_order: String,
    #[serde(default)]
    pub poster_position: Option<PosterPosition>,
    #[serde(default)]
    pub badge_style: Option<BadgeStyle>,
    #[serde(default = "db::default_label_style")]
    pub label_style: LabelStyle,
    #[serde(default = "db::default_poster_badge_direction")]
    pub badge_direction: BadgeDirection,
    #[serde(default = "db::default_badge_size")]
    pub badge_size: BadgeSize,
    #[serde(default, rename = "imageSize")]
    pub image_size: Option<String>,
}

/// Parse and validate the optional `imageSize` query parameter for previews.
fn parse_preview_image_size(
    raw: &Option<String>,
    kind: cache::ImageType,
) -> Result<Option<db::ImageSize>, AppError> {
    match raw {
        Some(s) => db::validate_image_size(s, kind).map(Some),
        None => Ok(None),
    }
}

fn default_ratings_limit() -> i32 {
    3
}

pub async fn preview_poster(
    State(state): State<Arc<AppState>>,
    Query(query): Query<PreviewQuery>,
) -> Result<Response, AppError> {
    let image_size = parse_preview_image_size(&query.image_size, cache::ImageType::Poster)?;
    let badge_size = query.badge_size;
    let resolved_size = serve::resolve_image_size(image_size);
    let target_width = resolved_size.poster_target_width();
    let badge_scale = resolved_size.badge_scale(cache::ImageType::Poster) * badge_size.scale_factor();

    let position = query.poster_position.unwrap_or(PosterPosition::BottomCenter);
    let raw_badge_style = query.badge_style.unwrap_or(BadgeStyle::Default);
    let label_style = query.label_style;
    let badge_direction = query.badge_direction.resolve(position);
    let badge_style = raw_badge_style.resolve(badge_direction);
    let suffix = ratings::ratings_cache_suffix(&query.ratings_order, query.ratings_limit);
    let pos_suffix = serve::poster_position_cache_suffix(position.as_str());
    let bs_suffix = serve::badge_style_cache_suffix(badge_style.as_str());
    let ls_suffix = serve::label_style_cache_suffix(label_style.as_str());
    let bd_suffix = serve::badge_direction_cache_suffix(badge_direction.as_str());
    let bsz_suffix = badge_size.cache_suffix();
    let is_suffix = serve::image_size_cache_suffix(image_size);
    let cache_key = format!("preview:{suffix}{pos_suffix}{bs_suffix}{ls_suffix}{bd_suffix}{bsz_suffix}{is_suffix}");
    let cache_path = cache::preview_path(&state.config.cache_dir, cache::ImageType::Poster, &format!("{suffix}{pos_suffix}{bs_suffix}{ls_suffix}{bd_suffix}{bsz_suffix}{is_suffix}"), "jpg")?;

    // 1. Check in-memory cache
    if let Some(cached) = state.preview_cache.get(&cache_key).await {
        return Ok(preview_response(cached));
    }

    // 2. Check filesystem cache (never stale — deterministic output)
    if let Some(entry) = cache::read(&cache_path, 0).await {
        let bytes: bytes::Bytes = entry.bytes.into();
        state.preview_cache.insert(cache_key, bytes.clone()).await;
        return Ok(preview_response(bytes));
    }

    // 3. Render and cache to both layers
    let badges = sample_badges();
    let badges = ratings::apply_rating_preferences(badges, &query.ratings_order, query.ratings_limit);

    let poster_png: &'static Vec<u8> = &SAMPLE_POSTER_PNG;
    let font = state.font.clone();
    let quality = state.config.image_quality;
    let buf = tokio::task::spawn_blocking(move || {
        generate::render_poster_sync(poster_png, &badges, &font, quality, position, badge_style, label_style, badge_direction, target_width, badge_scale, badge_size)
    })
    .await
    .map_err(|e| AppError::Other(e.to_string()))??;

    cache::write(&cache_path, &buf).await?;
    let bytes = bytes::Bytes::from(buf);
    state.preview_cache.insert(cache_key, bytes.clone()).await;

    Ok(preview_response(bytes))
}

pub async fn preview_logo(
    State(state): State<Arc<AppState>>,
    Query(query): Query<PreviewQuery>,
) -> Result<Response, AppError> {
    let image_size = parse_preview_image_size(&query.image_size, cache::ImageType::Logo)?;
    let badge_size = query.badge_size;
    let resolved_size = serve::resolve_image_size(image_size);
    let target_width = resolved_size.logo_target_width();
    let badge_scale = resolved_size.badge_scale(cache::ImageType::Logo) * badge_size.scale_factor();

    let badge_style = query.badge_style.unwrap_or(BadgeStyle::Horizontal);
    let label_style = query.label_style;
    let suffix = ratings::ratings_cache_suffix(&query.ratings_order, query.ratings_limit);
    let bs_suffix = serve::badge_style_cache_suffix(badge_style.as_str());
    let ls_suffix = serve::label_style_cache_suffix(label_style.as_str());
    let bsz_suffix = badge_size.cache_suffix();
    let is_suffix = serve::image_size_cache_suffix(image_size);
    let cache_key = format!("preview-logo:{suffix}{bs_suffix}{ls_suffix}{bsz_suffix}{is_suffix}");
    let cache_path = cache::preview_path(&state.config.cache_dir, cache::ImageType::Logo, &format!("{suffix}{bs_suffix}{ls_suffix}{bsz_suffix}{is_suffix}"), "png")?;

    if let Some(cached) = state.preview_cache.get(&cache_key).await {
        return Ok(preview_png_response(cached));
    }

    if let Some(entry) = cache::read(&cache_path, 0).await {
        let bytes: bytes::Bytes = entry.bytes.into();
        state.preview_cache.insert(cache_key, bytes.clone()).await;
        return Ok(preview_png_response(bytes));
    }

    let badges = sample_badges();
    let badges = ratings::apply_rating_preferences(badges, &query.ratings_order, query.ratings_limit);

    let logo_png: &'static Vec<u8> = &SAMPLE_LOGO_PNG;
    let font = state.font.clone();

    let buf = tokio::task::spawn_blocking(move || {
        generate::render_logo_sync(logo_png, &badges, &font, badge_style, label_style, target_width, badge_scale)
    })
    .await
    .map_err(|e| AppError::Other(e.to_string()))??;

    cache::write(&cache_path, &buf).await?;
    let bytes = bytes::Bytes::from(buf);
    state.preview_cache.insert(cache_key, bytes.clone()).await;

    Ok(preview_png_response(bytes))
}

pub async fn preview_backdrop(
    State(state): State<Arc<AppState>>,
    Query(query): Query<PreviewQuery>,
) -> Result<Response, AppError> {
    let image_size = parse_preview_image_size(&query.image_size, cache::ImageType::Backdrop)?;
    let badge_size = query.badge_size;
    let resolved_size = serve::resolve_image_size(image_size);
    let target_width = resolved_size.backdrop_target_width();
    let badge_scale = resolved_size.badge_scale(cache::ImageType::Backdrop) * badge_size.scale_factor();

    let badge_style = query.badge_style.unwrap_or(BadgeStyle::Vertical);
    let label_style = query.label_style;
    let suffix = ratings::ratings_cache_suffix(&query.ratings_order, query.ratings_limit);
    let bs_suffix = serve::badge_style_cache_suffix(badge_style.as_str());
    let ls_suffix = serve::label_style_cache_suffix(label_style.as_str());
    let bsz_suffix = badge_size.cache_suffix();
    let is_suffix = serve::image_size_cache_suffix(image_size);
    let cache_key = format!("preview-backdrop:{suffix}{bs_suffix}{ls_suffix}{bsz_suffix}{is_suffix}");
    let cache_path = cache::preview_path(&state.config.cache_dir, cache::ImageType::Backdrop, &format!("{suffix}{bs_suffix}{ls_suffix}{bsz_suffix}{is_suffix}"), "jpg")?;

    if let Some(cached) = state.preview_cache.get(&cache_key).await {
        return Ok(preview_response(cached));
    }

    if let Some(entry) = cache::read(&cache_path, 0).await {
        let bytes: bytes::Bytes = entry.bytes.into();
        state.preview_cache.insert(cache_key, bytes.clone()).await;
        return Ok(preview_response(bytes));
    }

    let badges = sample_badges();
    let badges = ratings::apply_rating_preferences(badges, &query.ratings_order, query.ratings_limit);

    let backdrop_png: &'static Vec<u8> = &SAMPLE_BACKDROP_PNG;
    let font = state.font.clone();
    let quality = state.config.image_quality;

    let buf = tokio::task::spawn_blocking(move || {
        generate::render_backdrop_sync(backdrop_png, &badges, &font, quality, badge_style, label_style, target_width, badge_scale)
    })
    .await
    .map_err(|e| AppError::Other(e.to_string()))??;

    cache::write(&cache_path, &buf).await?;
    let bytes = bytes::Bytes::from(buf);
    state.preview_cache.insert(cache_key, bytes.clone()).await;

    Ok(preview_response(bytes))
}

fn preview_response(bytes: bytes::Bytes) -> Response {
    (
        [
            (header::CONTENT_TYPE, "image/jpeg"),
            (header::CACHE_CONTROL, "public, max-age=60"),
        ],
        bytes,
    )
        .into_response()
}

fn preview_png_response(bytes: bytes::Bytes) -> Response {
    (
        [
            (header::CONTENT_TYPE, "image/png"),
            (header::CACHE_CONTROL, "public, max-age=60"),
        ],
        bytes,
    )
        .into_response()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sample_poster_png_is_valid() {
        let png = &*SAMPLE_POSTER_PNG;
        assert!(!png.is_empty());
        // PNG magic bytes
        assert_eq!(&png[..4], &[0x89, b'P', b'N', b'G']);
        // Should decode to 500x750
        let img = image::load_from_memory(png).expect("valid PNG");
        assert_eq!(img.width(), 500);
        assert_eq!(img.height(), 750);
    }

    #[test]
    fn sample_badges_returns_all_8_sources() {
        let badges = sample_badges();
        assert_eq!(badges.len(), 8);

        let sources: Vec<_> = badges.iter().map(|b| &b.source).collect();
        assert!(sources.contains(&&RatingSource::Imdb));
        assert!(sources.contains(&&RatingSource::Tmdb));
        assert!(sources.contains(&&RatingSource::Rt));
        assert!(sources.contains(&&RatingSource::RtAudience));
        assert!(sources.contains(&&RatingSource::Metacritic));
        assert!(sources.contains(&&RatingSource::Trakt));
        assert!(sources.contains(&&RatingSource::Letterboxd));
        assert!(sources.contains(&&RatingSource::Mal));
    }

    #[test]
    fn sample_poster_renders_with_badges() {
        let font = ab_glyph::FontArc::try_from_slice(crate::FONT_BYTES).unwrap();
        let badges = sample_badges();
        let result = generate::render_poster_sync(&SAMPLE_POSTER_PNG, &badges, &font, 85, PosterPosition::BottomCenter, BadgeStyle::Horizontal, LabelStyle::Text, BadgeDirection::Horizontal, 500, 1.0, BadgeSize::Medium);
        let buf = result.expect("rendering should succeed");
        // Valid JPEG
        assert_eq!(buf[0], 0xFF);
        assert_eq!(buf[1], 0xD8);
        assert!(buf.len() > 1000);
    }

    #[test]
    fn sample_poster_renders_with_no_badges() {
        let font = ab_glyph::FontArc::try_from_slice(crate::FONT_BYTES).unwrap();
        let result = generate::render_poster_sync(&SAMPLE_POSTER_PNG, &[], &font, 85, PosterPosition::BottomCenter, BadgeStyle::Horizontal, LabelStyle::Text, BadgeDirection::Horizontal, 500, 1.0, BadgeSize::Medium);
        let buf = result.expect("rendering should succeed");
        assert_eq!(buf[0], 0xFF);
        assert_eq!(buf[1], 0xD8);
    }

    #[test]
    fn default_ratings_limit_is_3() {
        assert_eq!(default_ratings_limit(), 3);
    }

    #[test]
    fn preview_query_defaults() {
        // Simulate what axum does with no query params — serde defaults apply
        let query: PreviewQuery = serde_json::from_str("{}").unwrap();
        assert_eq!(query.ratings_limit, 3);
        assert_eq!(query.ratings_order, "");
        assert_eq!(query.badge_style, None);
        assert_eq!(query.label_style, LabelStyle::Official);
        assert_eq!(query.badge_direction, BadgeDirection::Default);
        assert!(query.image_size.is_none());
    }

    #[test]
    fn preview_query_custom_values() {
        let query: PreviewQuery =
            serde_json::from_str(r#"{"ratings_limit":5,"ratings_order":"imdb,rt"}"#).unwrap();
        assert_eq!(query.ratings_limit, 5);
        assert_eq!(query.ratings_order, "imdb,rt");
    }
}
