use crate::error::AppError;
use serde::de::DeserializeOwned;

#[derive(Clone)]
pub struct TmdbClient {
    api_key: String,
    http: reqwest::Client,
}

impl TmdbClient {
    pub fn new(api_key: String, http: reqwest::Client) -> Self {
        Self { api_key, http }
    }

    pub async fn get<T: DeserializeOwned>(
        &self,
        path: &str,
        params: &[(&str, &str)],
    ) -> Result<T, AppError> {
        let url = format!("https://api.themoviedb.org/3{path}");
        let mut req = self.http.get(&url).query(&[("api_key", &self.api_key)]);
        if !params.is_empty() {
            req = req.query(params);
        }
        let resp = req.send().await?.error_for_status()?;
        Ok(resp.json().await?)
    }

    pub async fn fetch_poster_bytes(&self, poster_path: &str, http: &reqwest::Client) -> Result<Vec<u8>, AppError> {
        let url = format!("https://image.tmdb.org/t/p/w500{poster_path}");
        let resp = http.get(&url).send().await?.error_for_status()?;
        Ok(resp.bytes().await?.to_vec())
    }
}
