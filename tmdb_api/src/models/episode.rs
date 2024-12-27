use crate::utils::serialization::empty_string_as_none;
use chrono::NaiveDate;
use serde::Deserialize;

#[derive(Deserialize, Debug)]
pub struct Episode {
    pub episode_number: i32,
    pub name: Option<String>,
    pub still_path: Option<String>,
    pub overview: Option<String>,
    #[serde(default, deserialize_with = "empty_string_as_none")]
    pub air_date: Option<NaiveDate>,
}
