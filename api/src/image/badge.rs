use ab_glyph::{Font, FontArc, PxScale, ScaleFont};
use image::{imageops, Rgba, RgbaImage};
use imageproc::drawing::{draw_filled_rect_mut, draw_text_mut};
use imageproc::rect::Rect;

use crate::image::icons;
use crate::services::db::LabelStyle;
use crate::services::ratings::RatingBadge;

const DARK_BG: Rgba<u8> = Rgba([0, 0, 0, 200]);
const BASE_BADGE_HEIGHT: u32 = 58;
const BASE_BADGE_PADDING_H: u32 = 14;
const BASE_TEXT_LABEL_PADDING_H: u32 = 8;
const BASE_BADGE_VALUE_PADDING_H: u32 = 10;
const BASE_BADGE_RADIUS: u32 = 10;
const BASE_FONT_SIZE: f32 = 34.0;
const BASE_LABEL_FONT_SIZE: f32 = 26.0;
const BASE_ICON_HEIGHT: u32 = 48;

/// Compute the width of an icon when scaled to the given target height, preserving aspect ratio.
fn icon_scaled_width(icon: &RgbaImage, target_height: u32) -> u32 {
    if icon.height() == 0 {
        target_height
    } else {
        (icon.width() as f32 * target_height as f32 / icon.height() as f32).ceil() as u32
    }
}

/// Select the appropriate icon for a badge and compute its target (width, height) for the given
/// `label_style` and `icon_height`. For `Official`, the icon is fit within a square box;
/// otherwise it is scaled to the target height preserving aspect ratio.
fn badge_icon_and_size<'a>(badge: &'a RatingBadge, label_style: LabelStyle, icon_height: u32) -> (&'a RgbaImage, (u32, u32)) {
    let icon = match label_style {
        LabelStyle::Official => icons::official_icon_for_badge(badge),
        _ => icons::icon_for_source(&badge.source),
    };
    let dims = match label_style {
        LabelStyle::Official => icon_fit_in_box(icon, icon_height),
        _ => (icon_scaled_width(icon, icon_height), icon_height),
    };
    (icon, dims)
}

/// Compute dimensions to fit an icon within a `box_size x box_size` square, preserving aspect ratio.
fn icon_fit_in_box(icon: &RgbaImage, box_size: u32) -> (u32, u32) {
    let (w, h) = (icon.width(), icon.height());
    if w == 0 || h == 0 {
        return (box_size, box_size);
    }
    let scale = (box_size as f32 / w as f32).min(box_size as f32 / h as f32);
    (
        (w as f32 * scale).ceil() as u32,
        (h as f32 * scale).ceil() as u32,
    )
}

#[cfg(test)]
pub fn render_badge(badge: &RatingBadge, font: &FontArc, label_style: LabelStyle) -> RgbaImage {
    render_badge_with_widths(badge, font, None, None, label_style, 1.0)
}

/// Scaled badge dimensions for a given badge_scale factor.
struct ScaledDims {
    badge_height: u32,
    badge_padding_h: u32,
    text_label_padding_h: u32,
    badge_value_padding_h: u32,
    badge_radius: u32,
    icon_height: u32,
}

impl ScaledDims {
    fn new(badge_scale: f32) -> Self {
        Self {
            badge_height: (BASE_BADGE_HEIGHT as f32 * badge_scale).round() as u32,
            badge_padding_h: (BASE_BADGE_PADDING_H as f32 * badge_scale).round() as u32,
            text_label_padding_h: (BASE_TEXT_LABEL_PADDING_H as f32 * badge_scale).round() as u32,
            badge_value_padding_h: (BASE_BADGE_VALUE_PADDING_H as f32 * badge_scale).round() as u32,
            badge_radius: (BASE_BADGE_RADIUS as f32 * badge_scale).round() as u32,
            icon_height: (BASE_ICON_HEIGHT as f32 * badge_scale).round() as u32,
        }
    }
}

/// Pre-compute scaled fonts for badge rendering (avoids redundant work).
struct BadgeFonts<'a> {
    font: &'a FontArc,
    scale: PxScale,
    label_scale: PxScale,
    scaled: ab_glyph::PxScaleFont<&'a FontArc>,
    label_scaled: ab_glyph::PxScaleFont<&'a FontArc>,
}

impl<'a> BadgeFonts<'a> {
    fn new(font: &'a FontArc, badge_scale: f32) -> Self {
        let font_size = BASE_FONT_SIZE * badge_scale;
        let label_font_size = BASE_LABEL_FONT_SIZE * badge_scale;
        let scale = PxScale::from(font_size);
        let label_scale = PxScale::from(label_font_size);
        Self {
            font,
            scale,
            label_scale,
            scaled: font.as_scaled(scale),
            label_scaled: font.as_scaled(label_scale),
        }
    }
}

/// Render all badges with uniform label and value section widths.
pub fn render_badges_uniform(badges: &[RatingBadge], font: &FontArc, label_style: LabelStyle, badge_scale: f32) -> Vec<RgbaImage> {
    if badges.is_empty() {
        return vec![];
    }

    let fonts = BadgeFonts::new(font, badge_scale);
    let dims = ScaledDims::new(badge_scale);

    let max_label_width = match label_style {
        LabelStyle::Official => {
            // All official icons render within a fixed square box
            dims.icon_height
        }
        LabelStyle::Icon => {
            // For icon mode, use the max icon width (scaled to icon height)
            badges.iter()
                .map(|b| icon_scaled_width(icons::icon_for_source(&b.source), dims.icon_height))
                .max()
                .unwrap_or(dims.icon_height)
        }
        LabelStyle::Text => {
            badges.iter()
                .map(|b| text_width(b.source.label(), &fonts.label_scaled))
                .max()
                .unwrap_or(0)
        }
    };
    let max_value_width = badges.iter()
        .map(|b| text_width(&b.value, &fonts.scaled))
        .max()
        .unwrap_or(0);

    badges.iter()
        .map(|b| render_badge_inner(b, &fonts, &dims, Some(max_label_width), Some(max_value_width), label_style))
        .collect()
}

#[cfg(test)]
fn render_badge_with_widths(
    badge: &RatingBadge,
    font: &FontArc,
    uniform_label_width: Option<u32>,
    uniform_value_width: Option<u32>,
    label_style: LabelStyle,
    badge_scale: f32,
) -> RgbaImage {
    let fonts = BadgeFonts::new(font, badge_scale);
    let dims = ScaledDims::new(badge_scale);
    render_badge_inner(badge, &fonts, &dims, uniform_label_width, uniform_value_width, label_style)
}

fn render_badge_inner(
    badge: &RatingBadge,
    fonts: &BadgeFonts<'_>,
    dims: &ScaledDims,
    uniform_label_width: Option<u32>,
    uniform_value_width: Option<u32>,
    label_style: LabelStyle,
) -> RgbaImage {
    let use_icon = label_style.uses_icon();

    let label = badge.source.label();
    let value = &badge.value;

    let label_width = match label_style {
        LabelStyle::Official => {
            // Official icons all fit within a fixed square box
            uniform_label_width.unwrap_or(dims.icon_height)
        }
        LabelStyle::Icon => {
            let actual_w = icon_scaled_width(icons::icon_for_source(&badge.source), dims.icon_height);
            uniform_label_width.unwrap_or(actual_w)
        }
        LabelStyle::Text => {
            uniform_label_width.unwrap_or_else(|| text_width(label, &fonts.label_scaled))
        }
    };
    let value_width = uniform_value_width.unwrap_or_else(|| text_width(value, &fonts.scaled));
    let label_pad = if use_icon { dims.badge_padding_h } else { dims.text_label_padding_h };
    let total_width = label_width + value_width + label_pad * 2 + dims.badge_value_padding_h + dims.badge_value_padding_h / 2 + 2;

    let mut img = RgbaImage::new(total_width, dims.badge_height);

    // Draw label background (colored, or dark for official)
    let label_bg = match label_style {
        LabelStyle::Official => DARK_BG,
        _ => badge.source.color(),
    };
    draw_rounded_rect(&mut img, 0, 0, label_width + label_pad * 2, dims.badge_height, dims.badge_radius, label_bg);

    // Draw value background (dark)
    let value_x = label_width + label_pad * 2;
    draw_rounded_rect(&mut img, value_x, 0, total_width - value_x, dims.badge_height, dims.badge_radius, DARK_BG);

    // Overdraw the inner corners to make a clean join
    draw_filled_rect_mut(
        &mut img,
        Rect::at((label_width + label_pad) as i32, 0).of_size(label_pad, dims.badge_height),
        label_bg,
    );
    draw_filled_rect_mut(
        &mut img,
        Rect::at(value_x as i32, 0).of_size(label_pad, dims.badge_height),
        DARK_BG,
    );

    // Draw label (icon or text, centered within uniform label area)
    if use_icon {
        let (icon, (icon_w, icon_h)) = badge_icon_and_size(badge, label_style, dims.icon_height);
        let scaled_icon = if icon.width() == icon_w && icon.height() == icon_h {
            icon.clone()
        } else {
            imageops::resize(icon, icon_w, icon_h, imageops::FilterType::Lanczos3)
        };
        let ix = label_pad + (label_width.saturating_sub(icon_w)) / 2;
        let iy = (dims.badge_height.saturating_sub(icon_h)) / 2;
        imageops::overlay(&mut img, &scaled_icon, ix as i64, iy as i64);
    } else {
        let actual_label_width = text_width(label, &fonts.label_scaled);
        let label_x = label_pad + (label_width.saturating_sub(actual_label_width)) / 2;
        let label_y = (dims.badge_height as i32 - fonts.label_scale.x as i32) / 2;
        draw_text_mut(
            &mut img,
            Rgba([255, 255, 255, 255]),
            label_x as i32,
            label_y,
            fonts.label_scale,
            fonts.font,
            label,
        );
    }

    // Draw value text (centered within uniform value area)
    let actual_value_width = text_width(value, &fonts.scaled);
    let value_text_x = value_x + dims.badge_value_padding_h + (value_width.saturating_sub(actual_value_width)) / 2;
    let value_y = (dims.badge_height as i32 - fonts.scale.x as i32) / 2;
    draw_text_mut(
        &mut img,
        Rgba([255, 255, 255, 255]),
        value_text_x as i32,
        value_y,
        fonts.scale,
        fonts.font,
        value,
    );

    img
}

const BASE_VERT_BADGE_WIDTH: u32 = 88;
const BASE_VERT_BADGE_PADDING_V: u32 = 8;
const BASE_VERT_LABEL_FONT_SIZE: f32 = 26.0;
const BASE_VERT_VALUE_FONT_SIZE: f32 = 34.0;

/// Render a vertical badge: source label on top, rating value below.
/// Used for left/right poster positions.
pub fn render_vertical_badge(badge: &RatingBadge, font: &FontArc, label_style: LabelStyle, badge_scale: f32) -> RgbaImage {
    let use_icon = label_style.uses_icon();
    let vert_label_font_size = BASE_VERT_LABEL_FONT_SIZE * badge_scale;
    let vert_value_font_size = BASE_VERT_VALUE_FONT_SIZE * badge_scale;
    let label_scale = PxScale::from(vert_label_font_size);
    let value_scale = PxScale::from(vert_value_font_size);
    let vert_badge_width = (BASE_VERT_BADGE_WIDTH as f32 * badge_scale).round() as u32;
    let vert_badge_padding_v = (BASE_VERT_BADGE_PADDING_V as f32 * badge_scale).round() as u32;
    let icon_height = (BASE_ICON_HEIGHT as f32 * badge_scale).round() as u32;
    let badge_radius = (BASE_BADGE_RADIUS as f32 * badge_scale).round() as u32;

    let label = badge.source.label();
    let value = &badge.value;

    let label_area_h = if use_icon { icon_height } else { vert_label_font_size as u32 };
    let gap = (4.0 * badge_scale).round() as u32;
    let total_height = vert_badge_padding_v
        + label_area_h
        + gap
        + vert_value_font_size as u32
        + vert_badge_padding_v;

    let mut img = RgbaImage::new(vert_badge_width, total_height);

    // Draw full background with source color (or dark for official)
    let label_bg = match label_style {
        LabelStyle::Official => DARK_BG,
        _ => badge.source.color(),
    };
    draw_rounded_rect(&mut img, 0, 0, vert_badge_width, total_height, badge_radius, label_bg);

    // Draw a dark rect for the value area (bottom half)
    let value_area_y = vert_badge_padding_v + label_area_h + (gap / 2);
    let value_area_h = total_height - value_area_y;
    draw_rounded_rect(
        &mut img,
        0,
        value_area_y,
        vert_badge_width,
        value_area_h,
        badge_radius,
        DARK_BG,
    );
    // Overdraw the top corners of the dark area to clean the join
    draw_filled_rect_mut(
        &mut img,
        Rect::at(0, value_area_y as i32).of_size(vert_badge_width, badge_radius.min(value_area_h)),
        DARK_BG,
    );

    // Center label (icon or text) within the colored label area
    if use_icon {
        let (icon, (icon_w, icon_h)) = badge_icon_and_size(badge, label_style, icon_height);
        let scaled_icon = if icon.width() == icon_w && icon.height() == icon_h {
            icon.clone()
        } else {
            imageops::resize(icon, icon_w, icon_h, imageops::FilterType::Lanczos3)
        };
        let ix = (vert_badge_width.saturating_sub(icon_w)) / 2;
        let iy = (value_area_y.saturating_sub(icon_h)) / 2;
        imageops::overlay(&mut img, &scaled_icon, ix as i64, iy as i64);
    } else {
        let label_scaled_font = font.as_scaled(label_scale);
        let lw = text_width(label, &label_scaled_font);
        let label_x = (vert_badge_width.saturating_sub(lw)) / 2;
        let label_y = (value_area_y.saturating_sub(vert_label_font_size as u32)) / 2;
        draw_text_mut(
            &mut img,
            Rgba([255, 255, 255, 255]),
            label_x as i32,
            label_y as i32,
            label_scale,
            font,
            label,
        );
    }

    // Center value text
    let value_scaled_font = font.as_scaled(value_scale);
    let vw = text_width(value, &value_scaled_font);
    let value_x = (vert_badge_width.saturating_sub(vw)) / 2;
    let value_y = (value_area_y + vert_badge_padding_v / 2) as i32;
    draw_text_mut(
        &mut img,
        Rgba([255, 255, 255, 255]),
        value_x as i32,
        value_y,
        value_scale,
        font,
        value,
    );

    img
}

fn text_width(text: &str, font: &ab_glyph::PxScaleFont<&FontArc>) -> u32 {
    let width: f32 = text
        .chars()
        .map(|c| font.h_advance(font.glyph_id(c)))
        .sum();
    width.ceil() as u32
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::services::ratings::{RatingBadge, RatingSource};

    fn test_font() -> FontArc {
        FontArc::try_from_slice(crate::FONT_BYTES).unwrap()
    }

    #[test]
    fn render_badge_correct_height() {
        let badge = RatingBadge {
            source: RatingSource::Imdb,
            value: "8.5".to_string(),
        };
        let img = render_badge(&badge, &test_font(), LabelStyle::Text);
        assert_eq!(img.height(), BASE_BADGE_HEIGHT);
        assert!(img.width() > 0);
    }

    #[test]
    fn render_badge_all_sources_produce_valid_images() {
        let font = test_font();
        let sources = [
            RatingSource::Imdb,
            RatingSource::Tmdb,
            RatingSource::Rt,
            RatingSource::RtAudience,
            RatingSource::Metacritic,
            RatingSource::Trakt,
            RatingSource::Letterboxd,
            RatingSource::Mal,
        ];

        for source in sources {
            let badge = RatingBadge {
                source,
                value: "75%".to_string(),
            };
            let img = render_badge(&badge, &font, LabelStyle::Text);
            assert_eq!(img.height(), BASE_BADGE_HEIGHT, "wrong height for {:?}", source);
            assert!(img.width() > 0, "zero width for {:?}", source);
        }
    }

    #[test]
    fn render_badge_icon_all_sources() {
        let font = test_font();
        let sources = [
            RatingSource::Imdb,
            RatingSource::Tmdb,
            RatingSource::Rt,
            RatingSource::RtAudience,
            RatingSource::Metacritic,
            RatingSource::Trakt,
            RatingSource::Letterboxd,
            RatingSource::Mal,
        ];

        for source in sources {
            let badge = RatingBadge {
                source,
                value: "75%".to_string(),
            };
            let img = render_badge(&badge, &font, LabelStyle::Icon);
            assert_eq!(img.height(), BASE_BADGE_HEIGHT, "wrong height for {:?}", source);
            assert!(img.width() > 0, "zero width for {:?}", source);
        }
    }

    #[test]
    fn render_badge_width_scales_with_value_length() {
        let font = test_font();
        let short = RatingBadge {
            source: RatingSource::Imdb,
            value: "5".to_string(),
        };
        let long = RatingBadge {
            source: RatingSource::Imdb,
            value: "100%".to_string(),
        };

        let short_img = render_badge(&short, &font, LabelStyle::Text);
        let long_img = render_badge(&long, &font, LabelStyle::Text);

        assert!(
            long_img.width() > short_img.width(),
            "longer value should produce wider badge"
        );
    }

    #[test]
    fn render_vertical_badge_correct_dimensions() {
        let badge = RatingBadge {
            source: RatingSource::Imdb,
            value: "8.5".to_string(),
        };
        let img = render_vertical_badge(&badge, &test_font(), LabelStyle::Text, 1.0);
        assert_eq!(img.width(), BASE_VERT_BADGE_WIDTH);
        assert!(img.height() > 0);
    }

    #[test]
    fn render_vertical_badge_all_sources() {
        let font = test_font();
        let sources = [
            RatingSource::Imdb,
            RatingSource::Tmdb,
            RatingSource::Rt,
            RatingSource::RtAudience,
            RatingSource::Metacritic,
            RatingSource::Trakt,
            RatingSource::Letterboxd,
            RatingSource::Mal,
        ];

        for source in sources {
            let badge = RatingBadge {
                source,
                value: "75%".to_string(),
            };
            let img = render_vertical_badge(&badge, &font, LabelStyle::Text, 1.0);
            assert_eq!(img.width(), BASE_VERT_BADGE_WIDTH, "wrong width for {:?}", source);
            assert!(img.height() > 0, "zero height for {:?}", source);
        }
    }

    #[test]
    fn render_vertical_badge_icon_all_sources() {
        let font = test_font();
        let sources = [
            RatingSource::Imdb,
            RatingSource::Tmdb,
            RatingSource::Rt,
            RatingSource::RtAudience,
            RatingSource::Metacritic,
            RatingSource::Trakt,
            RatingSource::Letterboxd,
            RatingSource::Mal,
        ];

        for source in sources {
            let badge = RatingBadge {
                source,
                value: "75%".to_string(),
            };
            let img = render_vertical_badge(&badge, &font, LabelStyle::Icon, 1.0);
            assert_eq!(img.width(), BASE_VERT_BADGE_WIDTH, "wrong width for {:?}", source);
            assert!(img.height() > 0, "zero height for {:?}", source);
        }
    }

    #[test]
    fn render_badge_empty_value() {
        let font = test_font();
        let badge = RatingBadge {
            source: RatingSource::Tmdb,
            value: String::new(),
        };
        // Should not panic
        let img = render_badge(&badge, &font, LabelStyle::Text);
        assert_eq!(img.height(), BASE_BADGE_HEIGHT);
    }

    #[test]
    fn render_badge_scaled_2x_doubles_height() {
        let font = test_font();
        let badge = RatingBadge {
            source: RatingSource::Imdb,
            value: "8.5".to_string(),
        };
        let img = render_badge_with_widths(&badge, &font, None, None, LabelStyle::Text, 2.0);
        assert_eq!(img.height(), BASE_BADGE_HEIGHT * 2);
    }

    #[test]
    fn render_vertical_badge_scaled_2x_doubles_width() {
        let font = test_font();
        let badge = RatingBadge {
            source: RatingSource::Imdb,
            value: "8.5".to_string(),
        };
        let img = render_vertical_badge(&badge, &font, LabelStyle::Text, 2.0);
        assert_eq!(img.width(), BASE_VERT_BADGE_WIDTH * 2);
    }

    #[test]
    fn render_badges_uniform_scaled() {
        let font = test_font();
        let badges = vec![
            RatingBadge { source: RatingSource::Imdb, value: "8.5".to_string() },
            RatingBadge { source: RatingSource::Tmdb, value: "85%".to_string() },
        ];
        let images = render_badges_uniform(&badges, &font, LabelStyle::Text, 2.0);
        assert_eq!(images.len(), 2);
        // All badges should have doubled height
        for img in &images {
            assert_eq!(img.height(), BASE_BADGE_HEIGHT * 2);
        }
        // Uniform width: all badges same width
        assert_eq!(images[0].width(), images[1].width());
    }

    #[test]
    fn scaled_dims_at_1x() {
        let dims = ScaledDims::new(1.0);
        assert_eq!(dims.badge_height, BASE_BADGE_HEIGHT);
        assert_eq!(dims.badge_padding_h, BASE_BADGE_PADDING_H);
        assert_eq!(dims.text_label_padding_h, BASE_TEXT_LABEL_PADDING_H);
        assert_eq!(dims.badge_radius, BASE_BADGE_RADIUS);
        assert_eq!(dims.icon_height, BASE_ICON_HEIGHT);
    }
}

fn draw_rounded_rect(img: &mut RgbaImage, x: u32, y: u32, w: u32, h: u32, r: u32, color: Rgba<u8>) {
    // Simple approach: draw a filled rect and round corners by drawing circles
    // For simplicity, just draw the filled rect — true rounded rects need more complex logic
    draw_filled_rect_mut(
        img,
        Rect::at(x as i32, y as i32).of_size(w, h),
        color,
    );

    // Clear corners to simulate rounding (set to transparent)
    let transparent = Rgba([0, 0, 0, 0]);
    for dy in 0..r {
        for dx in 0..r {
            let dist_sq = (r - dx) * (r - dx) + (r - dy) * (r - dy);
            if dist_sq > r * r {
                // Top-left
                if x + dx < img.width() && y + dy < img.height() {
                    img.put_pixel(x + dx, y + dy, transparent);
                }
                // Top-right
                let rx = x + w - 1 - dx;
                if rx < img.width() && y + dy < img.height() {
                    img.put_pixel(rx, y + dy, transparent);
                }
                // Bottom-left
                let by = y + h - 1 - dy;
                if x + dx < img.width() && by < img.height() {
                    img.put_pixel(x + dx, by, transparent);
                }
                // Bottom-right
                if rx < img.width() && by < img.height() {
                    img.put_pixel(rx, by, transparent);
                }
            }
        }
    }
}
