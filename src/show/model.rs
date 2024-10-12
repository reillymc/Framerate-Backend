use std::env;

use crate::{
    season::{Season, SeasonResponse},
    utils::serialization::empty_string_as_none,
};
use chrono::NaiveDate;
use reqwest::header::AUTHORIZATION;
use serde::{Deserialize, Serialize};

use crate::error_handler::CustomError;

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all(serialize = "camelCase"))]
pub struct ExternalIds {
    pub imdb_id: Option<String>,
    pub tvdb_id: Option<i64>,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all(serialize = "camelCase"))]
pub struct ShowResponse {
    pub id: i32,
    pub name: String,
    pub poster_path: Option<String>,
    pub backdrop_path: Option<String>,
    #[serde(deserialize_with = "empty_string_as_none")]
    pub first_air_date: Option<NaiveDate>,
    pub overview: Option<String>,
    pub tagline: Option<String>,
    pub popularity: Option<f32>,
    pub external_ids: Option<ExternalIds>,
    pub seasons: Option<Vec<SeasonResponse>>,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all(serialize = "camelCase"))]
pub struct Show {
    pub id: i32,
    pub name: String,
    pub poster_path: Option<String>,
    pub backdrop_path: Option<String>,
    #[serde(deserialize_with = "empty_string_as_none")]
    pub first_air_date: Option<NaiveDate>,
    pub overview: Option<String>,
    pub tagline: Option<String>,
    pub popularity: Option<f32>,
    pub external_ids: Option<ExternalIds>,
    pub seasons: Option<Vec<Season>>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ShowSearchResults {
    pub page: i32,
    pub results: Vec<Show>,
}

impl Show {
    pub async fn find(id: &i32) -> Result<Show, CustomError> {
        let Ok(tbdb_api_key) = env::var("TMDB_API_KEY") else {
            return Err(CustomError::new(
                500,
                "TMDB API key must be set".to_string(),
            ));
        };

        let request_url = format!(
            "https://api.themoviedb.org/3/tv/{id}?language=en-AU&append_to_response=external_ids"
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

        let show = response.json::<ShowResponse>().await?;

        let seasons = if let Some(seasons) = &show.seasons {
            Some(
                seasons
                    .into_iter()
                    .map(|season| Season {
                        show_id: show.id,
                        season_number: season.season_number,
                        name: season.name.clone(),
                        overview: season.overview.clone(),
                        poster_path: season.poster_path.clone(),
                        air_date: season.air_date,
                        episode_count: season.episode_count,
                        episodes: None,
                    })
                    .collect::<Vec<Season>>(),
            )
        } else {
            None
        };

        let show = Show {
            id: show.id,
            name: show.name,
            overview: show.overview,
            tagline: show.tagline,
            popularity: show.popularity,
            external_ids: show.external_ids,
            backdrop_path: show.backdrop_path,
            first_air_date: show.first_air_date,
            poster_path: show.poster_path,
            seasons,
        };
        Ok(show)
    }

    pub async fn search(query: &str) -> Result<Vec<Show>, CustomError> {
        let Ok(tbdb_api_key) = env::var("TMDB_API_KEY") else {
            return Err(CustomError::new(
                500,
                "TMDB API key must be set".to_string(),
            ));
        };

        let request_url = format!(
            "https://api.themoviedb.org/3/search/tv?query={query}&include_adult=false&language=en-US&page=1"
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

        let search_results = response.json::<ShowSearchResults>().await?;
        Ok(search_results.results)
    }

    pub async fn popular() -> Result<Vec<Show>, CustomError> {
        let Ok(tbdb_api_key) = env::var("TMDB_API_KEY") else {
            return Err(CustomError::new(
                500,
                "TMDB API key must be set".to_string(),
            ));
        };

        let min_date = (chrono::Utc::now().date_naive() - chrono::Duration::days(10)).to_string();
        let max_date = (chrono::Utc::now().date_naive() + chrono::Duration::days(10)).to_string();

        let request_url = format!("https://api.themoviedb.org/3/discover/tv?include_adult=false&region=AU&include_video=false&language=en-US&page=1&sort_by=popularity.desc&with_original_language=en&air_date.gte={min_date}&air_date.lte={max_date}");

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

        let search_results = response.json::<ShowSearchResults>().await?;
        Ok(search_results.results)
    }
}
