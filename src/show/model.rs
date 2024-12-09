use crate::{
    error_handler::CustomError,
    season::{Episode, Season, SeasonResponse},
    tmdb::{generate_endpoint, TmdbClient},
    utils::serialization::empty_string_as_none,
};

use chrono::NaiveDate;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all(serialize = "camelCase"))]
pub struct ExternalIds {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub imdb_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tvdb_id: Option<i64>,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all(serialize = "camelCase"))]
pub struct ShowResponse {
    pub id: i32,
    pub name: String,
    pub poster_path: Option<String>,
    pub backdrop_path: Option<String>,
    #[serde(default, deserialize_with = "empty_string_as_none")]
    pub first_air_date: Option<NaiveDate>,
    #[serde(default, deserialize_with = "empty_string_as_none")]
    pub last_air_date: Option<NaiveDate>,
    pub last_episode_to_air: Option<Episode>,
    pub next_episode_to_air: Option<Episode>,
    pub status: Option<String>,
    pub overview: Option<String>,
    pub tagline: Option<String>,
    pub popularity: Option<f32>,
    pub external_ids: Option<ExternalIds>,
    pub seasons: Option<Vec<SeasonResponse>>,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all(serialize = "camelCase"))]
pub struct ShowSearchResponse {
    pub id: i32,
    pub name: String,
    pub poster_path: Option<String>,
    pub backdrop_path: Option<String>,
    #[serde(default, deserialize_with = "empty_string_as_none")]
    pub first_air_date: Option<NaiveDate>,
    pub status: Option<String>,
    pub overview: Option<String>,
    pub tagline: Option<String>,
    pub popularity: Option<f32>,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Show {
    pub id: i32,
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub poster_path: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub backdrop_path: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(default, deserialize_with = "empty_string_as_none")]
    pub first_air_date: Option<NaiveDate>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(default, deserialize_with = "empty_string_as_none")]
    pub last_air_date: Option<NaiveDate>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(default, deserialize_with = "empty_string_as_none")]
    pub next_air_date: Option<NaiveDate>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub status: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub overview: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tagline: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub popularity: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub external_ids: Option<ExternalIds>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub seasons: Option<Vec<Season>>,
}

pub const SHOW_ACTIVE_STATUSES: [&str; 4] =
    ["Returning Series", "Planned", "In Production", "Pilot"];

impl From<ShowSearchResponse> for Show {
    fn from(show: ShowSearchResponse) -> Self {
        Show {
            id: show.id,
            name: show.name,
            poster_path: show.poster_path,
            backdrop_path: show.backdrop_path,
            first_air_date: show.first_air_date,
            status: show.status,
            overview: show.overview,
            tagline: show.tagline,
            popularity: show.popularity,
            external_ids: None,
            seasons: None,
            last_air_date: None,
            next_air_date: None,
        }
    }
}

#[derive(Deserialize, Debug)]
struct ShowSearchResults {
    pub results: Vec<ShowSearchResponse>,
}

impl Show {
    pub async fn find(client: &TmdbClient, id: &i32) -> Result<Show, CustomError> {
        let request_url = generate_endpoint(
            format!("tv/{id}"),
            Some(HashMap::from([("append_to_response", "external_ids")])),
        );

        let response = client.get(&request_url).send().await?;

        if !response.status().is_success() {
            return Err(CustomError::new(
                response.status().as_u16(),
                response.text().await?.as_str(),
            ));
        }

        let show = response.json::<ShowResponse>().await?;

        let seasons = if let Some(seasons) = &show.seasons {
            Some(
                seasons
                    .iter()
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
                    .collect(),
            )
        } else {
            None
        };

        let next_air_date = if let Some(next_episode) = &show.next_episode_to_air {
            next_episode.air_date
        } else {
            None
        };

        let last_air_date = if let Some(last_episode) = &show.last_episode_to_air {
            last_episode.air_date
        } else {
            show.last_air_date
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
            last_air_date,
            next_air_date,
            status: show.status,
            poster_path: show.poster_path,
            seasons,
        };
        Ok(show)
    }

    pub async fn search(client: &TmdbClient, query: &str) -> Result<Vec<Show>, CustomError> {
        let request_url = generate_endpoint(
            format!("search/tv"),
            Some(HashMap::from([("query", query), ("page", "1")])),
        );

        let response = client.get(&request_url).send().await?;

        if !response.status().is_success() {
            return Err(CustomError::new(
                response.status().as_u16(),
                response.text().await?.as_str(),
            ));
        }

        let search_results = response.json::<ShowSearchResults>().await?;
        Ok(search_results
            .results
            .into_iter()
            .map(|show| show.into())
            .collect())
    }

    pub async fn popular(client: &TmdbClient) -> Result<Vec<Show>, CustomError> {
        let max_date = (chrono::Utc::now().date_naive() + chrono::Duration::weeks(26)).to_string();

        let request_url = generate_endpoint(
            format!("discover/tv"),
            Some(HashMap::from([
                ("air_date.lte", max_date.as_str()),
                ("page", "1"),
                ("region", "AU|NZ|US|XX"),
                ("show_me", "everything"),
                ("sort_by", "popularity.desc"),
                ("watch_region", "AU"),
                ("with_original_language", "en"),
                ("without_keywords", "210024"),
            ])),
        );

        let response = client.get(&request_url).send().await?;

        if !response.status().is_success() {
            return Err(CustomError::new(
                response.status().as_u16(),
                response.text().await?.as_str(),
            ));
        }

        let search_results = response.json::<ShowSearchResults>().await?;
        Ok(search_results
            .results
            .into_iter()
            .map(|show| show.into())
            .collect())
    }
}
