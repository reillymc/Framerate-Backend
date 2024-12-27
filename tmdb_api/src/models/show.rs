use super::{episode::Episode, season::Season};
use crate::utils::serialization::empty_string_as_none;
use chrono::NaiveDate;
use serde::Deserialize;

#[derive(Deserialize, Debug)]
pub struct Role {
    pub credit_id: String,
    pub character: Option<String>,
    pub episode_count: i64,
}

#[derive(Deserialize, Debug)]
pub struct Cast {
    pub id: i64,
    pub known_for_department: Option<String>,
    pub name: Option<String>,
    pub popularity: f64,
    pub profile_path: Option<String>,
    pub roles: Vec<Role>,
    pub total_episode_count: i64,
    pub order: i64,
}

#[derive(Deserialize, Debug)]
pub struct Job {
    pub credit_id: String,
    pub job: String,
    pub episode_count: i64,
}

#[derive(Deserialize, Debug)]
pub struct Crew {
    pub id: i64,
    pub known_for_department: Option<String>,
    pub name: Option<String>,
    pub popularity: f64,
    pub profile_path: Option<String>,
    pub jobs: Vec<Job>,
    pub department: Option<String>,
    pub total_episode_count: i64,
}

#[derive(Deserialize, Debug)]
pub struct Credits {
    pub cast: Vec<Cast>,
    pub crew: Vec<Crew>,
}

#[derive(Debug, Deserialize)]
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
    pub aggregate_credits: Option<Credits>,
}

#[derive(Deserialize, Debug)]
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
