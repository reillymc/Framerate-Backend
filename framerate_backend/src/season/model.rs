use crate::{
    tmdb::{generate_endpoint, TmdbClient},
    utils::AppError,
};
use chrono::NaiveDate;
use serde::{Deserialize, Serialize};
use tmdb_api::{episode, season, utils::serialization::empty_string_as_none};
use utoipa::ToSchema;

#[derive(ToSchema, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Episode {
    pub episode_number: i32,
    #[schema(nullable = false)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    #[schema(nullable = false)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub still_path: Option<String>,
    #[schema(nullable = false)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub overview: Option<String>,
    #[schema(nullable = false)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub air_date: Option<NaiveDate>,
}

#[derive(ToSchema, Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Season {
    pub show_id: i32,
    pub season_number: i32,
    #[schema(nullable = false)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    #[schema(nullable = false)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub poster_path: Option<String>,
    #[schema(nullable = false)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub overview: Option<String>,
    #[schema(nullable = false)]
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(default, deserialize_with = "empty_string_as_none")]
    pub air_date: Option<NaiveDate>,
    #[schema(nullable = false)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub episode_count: Option<i32>,
    #[schema(nullable = false)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub episodes: Option<Vec<Episode>>,
}

impl From<episode::Episode> for Episode {
    fn from(episode: episode::Episode) -> Self {
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
    ) -> Result<Season, AppError> {
        let request_url = generate_endpoint(format!("tv/{show_id}/season/{season_number}"), None);

        let response = client.get(&request_url).send().await?;

        if !response.status().is_success() {
            return Err(AppError::tmdb_error(
                response.status().as_u16(),
                response.text().await?.as_str(),
            ));
        }

        let season = response.json::<season::Season>().await?;

        let episodes = season
            .episodes
            .map(|episodes| episodes.into_iter().map(Episode::from).collect());

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
