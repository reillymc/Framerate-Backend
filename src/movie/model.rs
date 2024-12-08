use std::env;

use chrono::NaiveDate;
use reqwest::header::AUTHORIZATION;
use serde::{Deserialize, Serialize};

use crate::error_handler::CustomError;
use crate::utils::serialization::{date_time_as_date, empty_string_as_none};

#[derive(Deserialize, Debug)]
pub struct ReleaseDate {
    pub certification: Option<String>,
    #[serde(deserialize_with = "date_time_as_date")]
    pub release_date: Option<NaiveDate>,
    #[serde(rename = "type")]
    pub release_type: Option<i32>,
}

#[derive(Deserialize, Debug)]
pub struct ReleaseDateResult {
    pub iso_3166_1: String,
    pub release_dates: Vec<ReleaseDate>,
}

#[derive(Deserialize, Debug)]
pub struct ReleaseDates {
    pub results: Vec<ReleaseDateResult>,
}

#[derive(Deserialize)]
pub struct MovieResponse {
    pub id: i32,
    pub imdb_id: Option<String>,
    pub title: String,
    pub poster_path: Option<String>,
    pub backdrop_path: Option<String>,
    #[serde(default, deserialize_with = "empty_string_as_none")]
    pub release_date: Option<NaiveDate>,
    pub overview: Option<String>,
    pub tagline: Option<String>,
    pub popularity: Option<f32>,
    pub runtime: Option<i32>,
    pub release_dates: Option<ReleaseDates>,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Movie {
    pub id: i32,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub imdb_id: Option<String>,
    pub title: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub poster_path: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub backdrop_path: Option<String>,
    #[serde(default, deserialize_with = "empty_string_as_none")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub release_date: Option<NaiveDate>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub overview: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tagline: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub popularity: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub runtime: Option<i32>,
}

#[derive(Deserialize, Debug)]
pub struct MovieSearchResults {
    pub results: Vec<Movie>,
}

impl From<MovieResponse> for Movie {
    fn from(movie: MovieResponse) -> Self {
        let release_date = match &movie.release_dates {
            Some(release_dates) => {
                match release_dates
                    .results
                    .iter()
                    .find(|release_date| release_date.iso_3166_1 == "AU".to_string())
                {
                    Some(rel) => {
                        match rel
                            .release_dates
                            .iter()
                            .find(|release_date| release_date.release_type == Some(3))
                        {
                            Some(release) => release.release_date,
                            None => movie.release_date,
                        }
                    }
                    None => movie.release_date,
                }
            }
            None => movie.release_date,
        };

        Movie {
            id: movie.id,
            title: movie.title,
            poster_path: movie.poster_path,
            backdrop_path: movie.backdrop_path,
            release_date: release_date,
            overview: movie.overview,
            tagline: movie.tagline,
            popularity: movie.popularity,
            imdb_id: movie.imdb_id,
            runtime: movie.runtime,
        }
    }
}

impl Movie {
    pub async fn find(id: &i32) -> Result<Movie, CustomError> {
        let Ok(tbdb_api_key) = env::var("TMDB_API_KEY") else {
            return Err(CustomError::new(500, "TMDB API key must be set"));
        };

        let request_url = format!("https://api.themoviedb.org/3/movie/{id}?language=en-AU&append_to_response=release_dates");

        let client = reqwest::Client::new();

        let response = client
            .get(&request_url)
            .header(AUTHORIZATION, format!("Bearer {tbdb_api_key}"))
            .send()
            .await?;

        if !response.status().is_success() {
            return Err(CustomError::new(
                response.status().as_u16(),
                response.text().await?.as_str(),
            ));
        }

        let movie = response.json::<MovieResponse>().await?;
        Ok(movie.into())
    }

    pub async fn search(query: &str) -> Result<Vec<Movie>, CustomError> {
        let Ok(tbdb_api_key) = env::var("TMDB_API_KEY") else {
            return Err(CustomError::new(500, "TMDB API key must be set"));
        };

        let request_url = format!(
            "https://api.themoviedb.org/3/search/movie?query={query}&include_adult=false&language=en-US&region=AU&without_keywords=210024&page=1"
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
                response.text().await?.as_str(),
            ));
        }

        let search_results = response.json::<MovieSearchResults>().await?;
        Ok(search_results.results)
    }

    pub async fn popular() -> Result<Vec<Movie>, CustomError> {
        let Ok(tbdb_api_key) = env::var("TMDB_API_KEY") else {
            return Err(CustomError::new(500, "TMDB API key must be set"));
        };

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
                response.text().await?.as_str(),
            ));
        }

        let search_results = response.json::<MovieSearchResults>().await?;
        Ok(search_results.results)
    }
}
