use std::env;

use crate::{error_handler::CustomError, utils::serialization::empty_string_as_none};
use chrono::NaiveDate;
use reqwest::header::AUTHORIZATION;
use serde::{Deserialize, Serialize};

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all(serialize = "camelCase"))]
pub struct Episode {
    pub episode_number: i32,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub still_path: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub overview: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(deserialize_with = "empty_string_as_none")]
    pub air_date: Option<NaiveDate>,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all(serialize = "camelCase"))]
pub struct SeasonResponse {
    pub season_number: i32,
    pub name: Option<String>,
    pub poster_path: Option<String>,
    pub overview: Option<String>,
    #[serde(deserialize_with = "empty_string_as_none")]
    pub air_date: Option<NaiveDate>,
    pub episode_count: Option<i32>,
    pub episodes: Option<Vec<Episode>>,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all(serialize = "camelCase"))]
pub struct Season {
    pub show_id: i32,
    pub season_number: i32,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub poster_path: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub overview: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(deserialize_with = "empty_string_as_none")]
    pub air_date: Option<NaiveDate>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub episode_count: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub episodes: Option<Vec<Episode>>,
}

impl Season {
    pub async fn find(show_id: &i32, season_number: &i32) -> Result<Season, CustomError> {
        let Ok(tbdb_api_key) = env::var("TMDB_API_KEY") else {
            return Err(CustomError::new(
                500,
                "TMDB API key must be set".to_string(),
            ));
        };

        let request_url = format!(
            "https://api.themoviedb.org/3/tv/{show_id}/season/{season_number}?language=en-AU"
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

        let season = response.json::<SeasonResponse>().await?;

        let season = Season {
            name: season.name,
            overview: season.overview,
            air_date: season.air_date,
            episode_count: season.episode_count,
            episodes: season.episodes,
            poster_path: season.poster_path,
            season_number: season.season_number,
            show_id: *show_id,
        };

        Ok(season)
    }
}
