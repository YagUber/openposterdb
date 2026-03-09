use crate::id::{MediaType, ResolvedId};
use crate::services::omdb::OmdbClient;
use crate::services::tmdb::TmdbClient;
use image::Rgba;
use serde::Deserialize;

#[derive(Debug, Clone)]
pub struct RatingBadge {
    pub source: String,
    pub value: String,
    pub color: Rgba<u8>,
}

// Colors for each source
const IMDB_COLOR: Rgba<u8> = Rgba([245, 197, 24, 255]); // gold
const TMDB_COLOR: Rgba<u8> = Rgba([1, 210, 119, 255]); // green
const RT_COLOR: Rgba<u8> = Rgba([250, 50, 10, 255]); // red
const MC_COLOR: Rgba<u8> = Rgba([102, 204, 51, 255]); // metacritic green

pub async fn fetch_ratings(
    resolved: &ResolvedId,
    tmdb: &TmdbClient,
    omdb: &OmdbClient,
) -> Vec<RatingBadge> {
    let tmdb_fut = fetch_tmdb_rating(resolved, tmdb);
    let omdb_fut = fetch_omdb_ratings(resolved.imdb_id.as_deref(), omdb);

    let (tmdb_badges, omdb_badges) = tokio::join!(tmdb_fut, omdb_fut);

    let mut badges = Vec::new();

    // IMDb first (from OMDb), then TMDB, then RT, then MC
    if let Some(omdb_list) = omdb_badges {
        // Extract IMDb rating first
        for b in &omdb_list {
            if b.source == "IMDb" {
                badges.push(b.clone());
            }
        }
        // Add TMDB rating
        if let Some(tmdb_badge) = tmdb_badges {
            badges.push(tmdb_badge);
        }
        // Then RT and MC
        for b in omdb_list {
            if b.source != "IMDb" {
                badges.push(b);
            }
        }
    } else {
        if let Some(tmdb_badge) = tmdb_badges {
            badges.push(tmdb_badge);
        }
    }

    badges
}

async fn fetch_tmdb_rating(resolved: &ResolvedId, tmdb: &TmdbClient) -> Option<RatingBadge> {
    #[derive(Deserialize)]
    struct Details {
        vote_average: Option<f64>,
    }

    let path = match resolved.media_type {
        MediaType::Movie => format!("/movie/{}", resolved.tmdb_id),
        MediaType::Tv => format!("/tv/{}", resolved.tmdb_id),
    };

    let details: Details = tmdb.get(&path, &[]).await.ok()?;
    let score = details.vote_average?;
    if score <= 0.0 {
        return None;
    }

    Some(RatingBadge {
        source: "TMDB".to_string(),
        value: format!("{:.0}%", score * 10.0),
        color: TMDB_COLOR,
    })
}

async fn fetch_omdb_ratings(imdb_id: Option<&str>, omdb: &OmdbClient) -> Option<Vec<RatingBadge>> {
    let imdb_id = imdb_id?;
    let resp = omdb.get_ratings(imdb_id).await.ok()?;
    let mut badges = Vec::new();

    // IMDb rating
    if let Some(ref rating) = resp.imdb_rating {
        if rating != "N/A" {
            badges.push(RatingBadge {
                source: "IMDb".to_string(),
                value: rating.clone(),
                color: IMDB_COLOR,
            });
        }
    }

    // Rotten Tomatoes from Ratings array
    for r in &resp.ratings {
        if r.source == "Rotten Tomatoes" && r.value != "N/A" {
            badges.push(RatingBadge {
                source: "RT".to_string(),
                value: r.value.clone(),
                color: RT_COLOR,
            });
        }
    }

    // Metacritic
    if let Some(ref mc) = resp.metascore {
        if mc != "N/A" {
            badges.push(RatingBadge {
                source: "MC".to_string(),
                value: mc.clone(),
                color: MC_COLOR,
            });
        }
    }

    Some(badges)
}
