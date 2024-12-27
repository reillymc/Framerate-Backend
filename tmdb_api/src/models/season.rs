use super::episode::Episode;
use crate::utils::serialization::empty_string_as_none;
use chrono::NaiveDate;
use serde::Deserialize;

#[derive(Deserialize, Debug)]
pub struct Season {
    pub season_number: i32,
    pub name: Option<String>,
    pub poster_path: Option<String>,
    pub overview: Option<String>,
    #[serde(default, deserialize_with = "empty_string_as_none")]
    pub air_date: Option<NaiveDate>,
    pub episode_count: Option<i32>,
    pub episodes: Option<Vec<Episode>>,
}
