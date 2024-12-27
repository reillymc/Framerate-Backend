use super::{episode::Episode, season::Season};
use crate::utils::serialization::empty_string_as_none;
use chrono::NaiveDate;
use serde::Deserialize;

#[derive(Debug, Deserialize)]
#[serde(rename_all(serialize = "camelCase"))]
pub struct ExternalIds {
    pub imdb_id: Option<String>,
    pub tvdb_id: Option<i64>,
}

#[derive(Deserialize, Debug)]
pub struct Show {
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
    pub seasons: Option<Vec<Season>>,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all(serialize = "camelCase"))]
pub struct ShowSearch {
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
