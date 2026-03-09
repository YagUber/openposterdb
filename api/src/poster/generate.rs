use ab_glyph::FontArc;
use image::codecs::jpeg::JpegEncoder;
use image::{imageops, DynamicImage, RgbaImage};
use crate::cache;
use crate::error::AppError;
use crate::poster::badge;
use crate::services::ratings::RatingBadge;
use crate::services::tmdb::TmdbClient;

const BADGE_SPACING: u32 = 6;
const BADGE_BOTTOM_MARGIN: u32 = 12;

pub async fn generate_poster(
    poster_path: &str,
    badges: &[RatingBadge],
    tmdb: &TmdbClient,
    http: &reqwest::Client,
    font: &FontArc,
    quality: u8,
    cache_dir: &str,
    poster_stale_secs: u64,
) -> Result<Vec<u8>, AppError> {
    // Fetch base poster, using cache
    let poster_cache = cache::poster_cache_path(cache_dir, poster_path);
    let poster_bytes = if let Some(entry) = cache::read(&poster_cache, poster_stale_secs).await {
        if entry.is_stale {
            // Stale — refetch in foreground (rare with default 0 = never stale)
            let bytes = tmdb.fetch_poster_bytes(poster_path, http).await?;
            cache::write(&poster_cache, &bytes).await?;
            bytes
        } else {
            entry.bytes
        }
    } else {
        let bytes = tmdb.fetch_poster_bytes(poster_path, http).await?;
        cache::write(&poster_cache, &bytes).await?;
        bytes
    };

    let base = image::load_from_memory(&poster_bytes)
        .map_err(AppError::Image)?;
    let mut canvas: RgbaImage = base.to_rgba8();

    if !badges.is_empty() {
        // Render all badge images
        let badge_images: Vec<RgbaImage> = badges
            .iter()
            .map(|b| badge::render_badge(b, font))
            .collect();

        // Calculate total width of all badges
        let total_width: u32 = badge_images.iter().map(|b| b.width()).sum::<u32>()
            + BADGE_SPACING * (badge_images.len() as u32).saturating_sub(1);

        // Center badges at the bottom
        let start_x = (canvas.width().saturating_sub(total_width)) / 2;
        let y = canvas.height() - BADGE_BOTTOM_MARGIN - badge_images[0].height();

        let mut x = start_x;
        for badge_img in &badge_images {
            // Draw a semi-transparent backdrop behind the badge for readability
            imageops::overlay(&mut canvas, badge_img, x as i64, y as i64);
            x += badge_img.width() + BADGE_SPACING;
        }
    }

    // Encode as JPEG
    let dynamic = DynamicImage::ImageRgba8(canvas);
    let rgb = dynamic.to_rgb8();
    let mut buf = Vec::new();
    let encoder = JpegEncoder::new_with_quality(&mut buf, quality);
    rgb.write_with_encoder(encoder)?;

    Ok(buf)
}

/// Generate a 1x1 transparent placeholder JPEG
pub fn placeholder_jpeg() -> Vec<u8> {
    let img = image::RgbImage::from_pixel(1, 1, image::Rgb([0, 0, 0]));
    let mut buf = Vec::new();
    let encoder = JpegEncoder::new_with_quality(&mut buf, 50);
    img.write_with_encoder(encoder).ok();
    buf
}
