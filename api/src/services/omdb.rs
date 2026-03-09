use crate::error::AppError;
use serde::Deserialize;

#[derive(Clone)]
pub struct OmdbClient {
    api_key: String,
    http: reqwest::Client,
}

#[derive(Debug, Deserialize)]
pub struct OmdbResponse {
    #[serde(rename = "Ratings", default)]
    pub ratings: Vec<OmdbRating>,
    #[serde(rename = "imdbRating")]
    pub imdb_rating: Option<String>,
    #[serde(rename = "Metascore")]
    pub metascore: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct OmdbRating {
    #[serde(rename = "Source")]
    pub source: String,
    #[serde(rename = "Value")]
    pub value: String,
}

impl OmdbClient {
    pub fn new(api_key: String, http: reqwest::Client) -> Self {
        Self { api_key, http }
    }

    pub async fn get_ratings(&self, imdb_id: &str) -> Result<OmdbResponse, AppError> {
        let resp = self
            .http
            .get("https://www.omdbapi.com/")
            .query(&[("apikey", &self.api_key), ("i", &imdb_id.to_string())])
            .send()
            .await?
            .error_for_status()?;
        Ok(resp.json().await?)
    }
}
