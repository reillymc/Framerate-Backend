use crate::{
    error_handler::CustomError,
    tmdb::{generate_endpoint, TmdbClient},
    utils::serialization::empty_string_as_none,
};
use chrono::NaiveDate;
use serde::{Deserialize, Serialize};

#[derive(Default, Debug, Clone, PartialEq, Deserialize)]
pub struct EpisodeResponse {
    pub episode_number: i32,
    pub name: Option<String>,
    pub still_path: Option<String>,
    pub overview: Option<String>,
    #[serde(default, deserialize_with = "empty_string_as_none")]
    pub air_date: Option<NaiveDate>,
}

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
    pub air_date: Option<NaiveDate>,
}

#[derive(Deserialize, Debug)]
pub struct SeasonResponse {
    pub season_number: i32,
    pub name: Option<String>,
    pub poster_path: Option<String>,
    pub overview: Option<String>,
    #[serde(default, deserialize_with = "empty_string_as_none")]
    pub air_date: Option<NaiveDate>,
    pub episode_count: Option<i32>,
    pub episodes: Option<Vec<EpisodeResponse>>,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
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
    #[serde(default, deserialize_with = "empty_string_as_none")]
    pub air_date: Option<NaiveDate>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub episode_count: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub episodes: Option<Vec<Episode>>,
}

impl From<EpisodeResponse> for Episode {
    fn from(episode: EpisodeResponse) -> Self {
        Episode {
            episode_number: episode.episode_number,
            name: episode.name,
            still_path: episode.still_path,
            overview: episode.overview,
            air_date: episode.air_date,
        }
    }
}

impl Season {
    pub async fn find(
        client: &TmdbClient,
        show_id: &i32,
        season_number: &i32,
    ) -> Result<Season, CustomError> {
        let request_url = generate_endpoint(format!("tv/{show_id}/season/{season_number}"), None);

        let response = client.get(&request_url).send().await?;

        if !response.status().is_success() {
            return Err(CustomError::new(
                response.status().as_u16(),
                response.text().await?.as_str(),
            ));
        }

        let season = response.json::<SeasonResponse>().await?;

        let episodes = if let Some(episodes) = season.episodes {
            Some(episodes.into_iter().map(Episode::from).collect())
        } else {
            None
        };

        let season = Season {
            name: season.name,
            overview: season.overview,
            air_date: season.air_date,
            episode_count: season.episode_count,
            episodes,
            poster_path: season.poster_path,
            season_number: season.season_number,
            show_id: *show_id,
        };

        Ok(season)
    }
}
