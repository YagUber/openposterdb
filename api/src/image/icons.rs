use image::RgbaImage;
use std::sync::LazyLock;

use crate::services::ratings::{RatingBadge, RatingSource};

static IMDB_BYTES: &[u8] = include_bytes!("../../assets/icons/imdb.png");
static TMDB_BYTES: &[u8] = include_bytes!("../../assets/icons/tmdb.png");
static RT_BYTES: &[u8] = include_bytes!("../../assets/icons/rt.png");
static RTA_BYTES: &[u8] = include_bytes!("../../assets/icons/rta.png");
static MC_BYTES: &[u8] = include_bytes!("../../assets/icons/mc.png");
static TRAKT_BYTES: &[u8] = include_bytes!("../../assets/icons/trakt.png");
static LB_BYTES: &[u8] = include_bytes!("../../assets/icons/lb.png");
static MAL_BYTES: &[u8] = include_bytes!("../../assets/icons/mal.png");

// Official icons
static OFF_IMDB_BYTES: &[u8] = include_bytes!("../../assets/icons/official/imdb.png");
static OFF_TMDB_BYTES: &[u8] = include_bytes!("../../assets/icons/official/tmdb.png");
static OFF_MC_BYTES: &[u8] = include_bytes!("../../assets/icons/official/metacritic.png");
static OFF_TRAKT_BYTES: &[u8] = include_bytes!("../../assets/icons/official/trakt.png");
static OFF_LB_BYTES: &[u8] = include_bytes!("../../assets/icons/official/letterboxd.png");
static OFF_MAL_BYTES: &[u8] = include_bytes!("../../assets/icons/official/mal.webp");
static OFF_RT_CRITIC_POSITIVE_BYTES: &[u8] = include_bytes!("../../assets/icons/official/Rotten_Tomatoes_critic_positive.png");
static OFF_RT_CRITIC_ROTTEN_BYTES: &[u8] = include_bytes!("../../assets/icons/official/Rotten_Tomatoes_critic_rotten.png");
static OFF_RT_CRITIC_CERTIFIED_FRESH_BYTES: &[u8] = include_bytes!("../../assets/icons/official/Rotten_Tomatoes_critic_certified_fresh.png");
static OFF_RT_AUDIENCE_POSITIVE_BYTES: &[u8] = include_bytes!("../../assets/icons/official/Rotten_Tomatoes_positive_audience.png");
static OFF_RT_AUDIENCE_NEGATIVE_BYTES: &[u8] = include_bytes!("../../assets/icons/official/Rotten_Tomatoes_negative_audience.png");
static OFF_RT_AUDIENCE_VERIFIED_HOT_BYTES: &[u8] = include_bytes!("../../assets/icons/official/Rotten_Tomatoes_verified_hot_audience.png");

fn decode(bytes: &[u8]) -> RgbaImage {
    image::load_from_memory(bytes)
        .expect("embedded icon should be valid")
        .to_rgba8()
}

static IMDB_IMG: LazyLock<RgbaImage> = LazyLock::new(|| decode(IMDB_BYTES));
static TMDB_IMG: LazyLock<RgbaImage> = LazyLock::new(|| decode(TMDB_BYTES));
static RT_IMG: LazyLock<RgbaImage> = LazyLock::new(|| decode(RT_BYTES));
static RTA_IMG: LazyLock<RgbaImage> = LazyLock::new(|| decode(RTA_BYTES));
static MC_IMG: LazyLock<RgbaImage> = LazyLock::new(|| decode(MC_BYTES));
static TRAKT_IMG: LazyLock<RgbaImage> = LazyLock::new(|| decode(TRAKT_BYTES));
static LB_IMG: LazyLock<RgbaImage> = LazyLock::new(|| decode(LB_BYTES));
static MAL_IMG: LazyLock<RgbaImage> = LazyLock::new(|| decode(MAL_BYTES));

// Official icon images
static OFF_IMDB_IMG: LazyLock<RgbaImage> = LazyLock::new(|| decode(OFF_IMDB_BYTES));
static OFF_TMDB_IMG: LazyLock<RgbaImage> = LazyLock::new(|| decode(OFF_TMDB_BYTES));
static OFF_MC_IMG: LazyLock<RgbaImage> = LazyLock::new(|| decode(OFF_MC_BYTES));
static OFF_TRAKT_IMG: LazyLock<RgbaImage> = LazyLock::new(|| decode(OFF_TRAKT_BYTES));
static OFF_LB_IMG: LazyLock<RgbaImage> = LazyLock::new(|| decode(OFF_LB_BYTES));
static OFF_MAL_IMG: LazyLock<RgbaImage> = LazyLock::new(|| decode(OFF_MAL_BYTES));
static OFF_RT_CRITIC_POSITIVE_IMG: LazyLock<RgbaImage> = LazyLock::new(|| decode(OFF_RT_CRITIC_POSITIVE_BYTES));
static OFF_RT_CRITIC_ROTTEN_IMG: LazyLock<RgbaImage> = LazyLock::new(|| decode(OFF_RT_CRITIC_ROTTEN_BYTES));
static OFF_RT_CRITIC_CERTIFIED_FRESH_IMG: LazyLock<RgbaImage> = LazyLock::new(|| decode(OFF_RT_CRITIC_CERTIFIED_FRESH_BYTES));
static OFF_RT_AUDIENCE_POSITIVE_IMG: LazyLock<RgbaImage> = LazyLock::new(|| decode(OFF_RT_AUDIENCE_POSITIVE_BYTES));
static OFF_RT_AUDIENCE_NEGATIVE_IMG: LazyLock<RgbaImage> = LazyLock::new(|| decode(OFF_RT_AUDIENCE_NEGATIVE_BYTES));
static OFF_RT_AUDIENCE_VERIFIED_HOT_IMG: LazyLock<RgbaImage> = LazyLock::new(|| decode(OFF_RT_AUDIENCE_VERIFIED_HOT_BYTES));

pub fn icon_for_source(source: &RatingSource) -> &'static RgbaImage {
    match source {
        RatingSource::Imdb => &IMDB_IMG,
        RatingSource::Tmdb => &TMDB_IMG,
        RatingSource::Rt => &RT_IMG,
        RatingSource::RtAudience => &RTA_IMG,
        RatingSource::Metacritic => &MC_IMG,
        RatingSource::Trakt => &TRAKT_IMG,
        RatingSource::Letterboxd => &LB_IMG,
        RatingSource::Mal => &MAL_IMG,
    }
}

fn parse_percent(value: &str) -> Option<u32> {
    let s = value.trim_end_matches('%');
    // Try integer first, then float (e.g. "8.0" → 8)
    s.parse::<u32>()
        .ok()
        .or_else(|| s.parse::<f32>().ok().map(|f| f as u32))
}

pub fn official_icon_for_badge(badge: &RatingBadge) -> &'static RgbaImage {
    match badge.source {
        RatingSource::Imdb => &OFF_IMDB_IMG,
        RatingSource::Tmdb => &OFF_TMDB_IMG,
        RatingSource::Metacritic => &OFF_MC_IMG,
        RatingSource::Trakt => &OFF_TRAKT_IMG,
        RatingSource::Letterboxd => &OFF_LB_IMG,
        RatingSource::Mal => &OFF_MAL_IMG,
        RatingSource::Rt => {
            let score = parse_percent(&badge.value).unwrap_or(0);
            if score >= 75 {
                &OFF_RT_CRITIC_CERTIFIED_FRESH_IMG
            } else if score >= 60 {
                &OFF_RT_CRITIC_POSITIVE_IMG
            } else {
                &OFF_RT_CRITIC_ROTTEN_IMG
            }
        }
        RatingSource::RtAudience => {
            let score = parse_percent(&badge.value).unwrap_or(0);
            if score >= 75 {
                &OFF_RT_AUDIENCE_VERIFIED_HOT_IMG
            } else if score >= 60 {
                &OFF_RT_AUDIENCE_POSITIVE_IMG
            } else {
                &OFF_RT_AUDIENCE_NEGATIVE_IMG
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn all_icons_decode_to_48x48() {
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
            let img = icon_for_source(&source);
            assert_eq!(img.width(), 48, "wrong width for {:?}", source);
            assert_eq!(img.height(), 48, "wrong height for {:?}", source);
        }
    }

    #[test]
    fn all_official_icons_fit_48x48() {
        let badges = [
            RatingBadge { source: RatingSource::Imdb, value: "8.5".into() },
            RatingBadge { source: RatingSource::Tmdb, value: "85%".into() },
            RatingBadge { source: RatingSource::Metacritic, value: "80".into() },
            RatingBadge { source: RatingSource::Trakt, value: "90%".into() },
            RatingBadge { source: RatingSource::Letterboxd, value: "4.0".into() },
            RatingBadge { source: RatingSource::Mal, value: "8.5".into() },
            // RT critics — all three variants
            RatingBadge { source: RatingSource::Rt, value: "95%".into() },
            RatingBadge { source: RatingSource::Rt, value: "65%".into() },
            RatingBadge { source: RatingSource::Rt, value: "40%".into() },
            // RT audience — all three variants
            RatingBadge { source: RatingSource::RtAudience, value: "90%".into() },
            RatingBadge { source: RatingSource::RtAudience, value: "65%".into() },
            RatingBadge { source: RatingSource::RtAudience, value: "40%".into() },
        ];
        for badge in &badges {
            let img = official_icon_for_badge(badge);
            assert!(img.width() <= 48, "width {} > 48 for {:?} with value {}", img.width(), badge.source, badge.value);
            assert!(img.height() <= 48, "height {} > 48 for {:?} with value {}", img.height(), badge.source, badge.value);
            assert!(img.width() == 48 || img.height() == 48, "neither dimension is 48 for {:?} with value {}", badge.source, badge.value);
        }
    }

    #[test]
    fn parse_percent_works() {
        assert_eq!(parse_percent("95%"), Some(95));
        assert_eq!(parse_percent("8.0"), Some(8));
        assert_eq!(parse_percent("72"), Some(72));
        assert_eq!(parse_percent(""), None);
        assert_eq!(parse_percent("N/A"), None);
    }

    #[test]
    fn rt_critic_icon_thresholds() {
        let cf = RatingBadge { source: RatingSource::Rt, value: "75%".into() };
        let fresh = RatingBadge { source: RatingSource::Rt, value: "60%".into() };
        let rotten = RatingBadge { source: RatingSource::Rt, value: "59%".into() };

        assert!(std::ptr::eq(official_icon_for_badge(&cf), &*OFF_RT_CRITIC_CERTIFIED_FRESH_IMG));
        assert!(std::ptr::eq(official_icon_for_badge(&fresh), &*OFF_RT_CRITIC_POSITIVE_IMG));
        assert!(std::ptr::eq(official_icon_for_badge(&rotten), &*OFF_RT_CRITIC_ROTTEN_IMG));
    }

    #[test]
    fn rt_audience_icon_thresholds() {
        let hot = RatingBadge { source: RatingSource::RtAudience, value: "75%".into() };
        let pos = RatingBadge { source: RatingSource::RtAudience, value: "60%".into() };
        let neg = RatingBadge { source: RatingSource::RtAudience, value: "59%".into() };

        assert!(std::ptr::eq(official_icon_for_badge(&hot), &*OFF_RT_AUDIENCE_VERIFIED_HOT_IMG));
        assert!(std::ptr::eq(official_icon_for_badge(&pos), &*OFF_RT_AUDIENCE_POSITIVE_IMG));
        assert!(std::ptr::eq(official_icon_for_badge(&neg), &*OFF_RT_AUDIENCE_NEGATIVE_IMG));
    }
}
