use std::env;

use chrono::NaiveDate;
use reqwest::header::AUTHORIZATION;
use serde::{Deserialize, Serialize};

use crate::error_handler::CustomError;

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all(serialize = "camelCase"))]
pub struct Movie {
    pub id: i32,
    pub imdb_id: Option<String>,
    pub title: String,
    pub poster_path: Option<String>,
    pub backdrop_path: Option<String>,
    pub release_date: Option<NaiveDate>,
    pub overview: Option<String>,
    pub tagline: Option<String>,
    pub popularity: f32,
    pub runtime: i32,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all(serialize = "camelCase"))]
pub struct MovieSearchResult {
    pub id: i32,
    pub title: String,
    pub poster_path: Option<String>,
    pub backdrop_path: Option<String>,
    pub release_date: Option<NaiveDate>,
    pub overview: Option<String>,
    pub popularity: f32,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct MovieSearchResults {
    pub page: i32,
    pub results: Vec<MovieSearchResult>,
}

impl Movie {
    pub async fn find(id: i32) -> Result<Movie, CustomError> {
        let tbdb_api_key = env::var("TMDB_API_KEY").expect("TMDB API key must be set");

        let request_url = format!("https://api.themoviedb.org/3/movie/{id}?language=en-AU");

        let client = reqwest::Client::new();

        let response = client
            .get(&request_url)
            .header(AUTHORIZATION, format!("Bearer {tbdb_api_key}"))
            .send()
            .await?;

        if !response.status().is_success() {
            return Err(CustomError::new(
                response.status().as_u16(),
                response.text().await?,
            ));
        }

        let movie = response.json::<Movie>().await?;
        Ok(movie)
    }

    pub async fn search(query: &str) -> Result<Vec<MovieSearchResult>, CustomError> {
        let tbdb_api_key = env::var("TMDB_API_KEY").expect("TMDB API key must be set");

        let request_url = format!(
            "https://api.themoviedb.org/3/search/movie?query={query}&include_adult=false&language=en-US&page=1"
        );

        let client = reqwest::Client::new();

        let response = client
            .get(&request_url)
            .header(AUTHORIZATION, format!("Bearer {tbdb_api_key}"))
            .send()
            .await?;

        if !response.status().is_success() {
            return Err(CustomError::new(
                response.status().as_u16(),
                response.text().await?,
            ));
        }

        let search_results = response.json::<MovieSearchResults>().await?;
        Ok(search_results.results)
    }

    pub async fn popular() -> Result<Vec<MovieSearchResult>, CustomError> {
        let tbdb_api_key = env::var("TMDB_API_KEY").expect("TMDB API key must be set");

        let min_date = (chrono::Utc::now().date_naive() - chrono::Duration::days(30)).to_string();
        let max_date = (chrono::Utc::now().date_naive() + chrono::Duration::days(7)).to_string();

        let request_url = format!("https://api.themoviedb.org/3/discover/movie?include_adult=false&region=AU&include_video=false&language=en-US&page=1&sort_by=popularity.desc&with_release_type=2|3&release_date.gte={min_date}&release_date.lte={max_date}");

        let client = reqwest::Client::new();

        let response = client
            .get(&request_url)
            .header(AUTHORIZATION, format!("Bearer {tbdb_api_key}"))
            .send()
            .await?;

        if !response.status().is_success() {
            return Err(CustomError::new(
                response.status().as_u16(),
                response.text().await?,
            ));
        }

        let search_results = response.json::<MovieSearchResults>().await?;
        Ok(search_results.results)
    }
}
