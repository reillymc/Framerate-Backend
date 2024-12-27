use chrono::NaiveDate;
use serde::Deserialize;

use crate::utils::serialization::{date_time_as_date, empty_string_as_none};

#[derive(Deserialize, Debug)]
pub struct Cast {
    pub id: i64,
    pub known_for_department: Option<String>,
    pub name: Option<String>,
    pub popularity: f64,
    pub profile_path: Option<String>,
    pub cast_id: i64,
    pub character: Option<String>,
    pub credit_id: Option<String>,
    pub order: i64,
}

#[derive(Deserialize, Debug)]
pub struct Crew {
    pub id: i64,
    pub known_for_department: Option<String>,
    pub name: Option<String>,
    pub popularity: f64,
    pub profile_path: Option<String>,
    pub credit_id: Option<String>,
    pub department: Option<String>,
    pub job: Option<String>,
}

#[derive(Deserialize, Debug)]
pub struct Credits {
    pub cast: Vec<Cast>,
    pub crew: Vec<Crew>,
}

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
pub struct Movie {
    pub id: i32,
    pub imdb_id: Option<String>,
    pub title: String,
    pub poster_path: Option<String>,
    pub backdrop_path: Option<String>,
    #[serde(default, deserialize_with = "empty_string_as_none")]
    pub release_date: Option<NaiveDate>,
    pub status: Option<String>,
    pub overview: Option<String>,
    pub tagline: Option<String>,
    pub popularity: Option<f32>,
    pub runtime: Option<i32>,
    pub release_dates: Option<ReleaseDates>,
    pub credits: Option<Credits>,
}

#[derive(Deserialize)]
pub struct MovieSearch {
    pub results: Vec<Movie>,
}
