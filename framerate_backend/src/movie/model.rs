use crate::tmdb::{generate_endpoint, TmdbClient};
use crate::utils::AppError;

use chrono::NaiveDate;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tmdb_api::{credits, movie, utils::serialization::empty_string_as_none};

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Cast {
    pub id: i64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub known_for_department: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    pub popularity: f64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub profile_path: Option<String>,
    pub cast_id: i64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub character: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub credit_id: Option<String>,
}

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Crew {
    pub id: i64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub known_for_department: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    pub popularity: f64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub profile_path: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub credit_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub department: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub job: Option<String>,
}

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Credits {
    pub cast: Vec<Cast>,
    pub crew: Vec<Crew>,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Movie {
    pub id: i32,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub imdb_id: Option<String>,
    pub title: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub poster_path: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub backdrop_path: Option<String>,
    #[serde(default, deserialize_with = "empty_string_as_none")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub release_date: Option<NaiveDate>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub status: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub overview: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tagline: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub popularity: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub runtime: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub credits: Option<Credits>,
}

pub const MOVIE_ACTIVE_STATUSES: [&str; 4] =
    ["Rumored", "Planned", "In Production", "Post Production"];

impl From<credits::Crew> for Crew {
    fn from(cast: credits::Crew) -> Self {
        Crew {
            credit_id: cast.credit_id,
            id: cast.id,
            known_for_department: cast.known_for_department,
            name: cast.name,
            popularity: cast.popularity,
            profile_path: cast.profile_path,
            department: cast.department,
            job: cast.job,
        }
    }
}
impl From<credits::Cast> for Cast {
    fn from(cast: credits::Cast) -> Self {
        Cast {
            cast_id: cast.cast_id,
            character: cast.character,
            credit_id: cast.credit_id,
            id: cast.id,
            known_for_department: cast.known_for_department,
            name: cast.name,
            popularity: cast.popularity,
            profile_path: cast.profile_path,
        }
    }
}

impl From<credits::Credits> for Credits {
    fn from(credits: credits::Credits) -> Self {
        let mut cast = credits.cast;
        cast.sort_by(|a, b| a.order.cmp(&b.order));
        let cast = cast.into_iter().map(Cast::from).collect();

        let crew = credits.crew.into_iter().map(Crew::from).collect();

        Credits { cast, crew }
    }
}

impl From<movie::Movie> for Movie {
    fn from(movie: movie::Movie) -> Self {
        let release_date = match &movie.release_dates {
            Some(release_dates) => {
                match release_dates
                    .results
                    .iter()
                    .find(|release_date| release_date.iso_3166_1 == *"AU")
                {
                    Some(rel) => {
                        match rel
                            .release_dates
                            .iter()
                            .find(|release_date| release_date.release_type == Some(3))
                        {
                            Some(release) => release.release_date,
                            None => movie.release_date,
                        }
                    }
                    None => movie.release_date,
                }
            }
            None => movie.release_date,
        };

        let credits = if let Some(credits) = movie.credits {
            Some(Credits::from(credits))
        } else {
            None
        };

        Movie {
            id: movie.id,
            title: movie.title,
            poster_path: movie.poster_path,
            backdrop_path: movie.backdrop_path,
            release_date,
            status: movie.status,
            overview: movie.overview,
            tagline: movie.tagline,
            popularity: movie.popularity,
            imdb_id: movie.imdb_id,
            runtime: movie.runtime,
            credits,
        }
    }
}

impl Movie {
    pub async fn find(client: &TmdbClient, id: &i32) -> Result<Movie, AppError> {
        let request_url = generate_endpoint(
            format!("movie/{id}"),
            Some(HashMap::from([(
                "append_to_response",
                "release_dates,credits",
            )])),
        );

        let response = client.get(&request_url).send().await?;

        if !response.status().is_success() {
            return Err(AppError::tmdb_error(
                response.status().as_u16(),
                response.text().await?.as_str(),
            ));
        }

        let movie = response.json::<movie::Movie>().await?;
        Ok(Movie::from(movie))
    }

    pub async fn search(client: &TmdbClient, query: &str) -> Result<Vec<Movie>, AppError> {
        let request_url = generate_endpoint(
            "search/movie".to_string(),
            Some(HashMap::from([
                ("query", query),
                ("region", "AU"),
                ("without_keywords", "210024"),
                ("page", "1"),
            ])),
        );

        let response = client.get(&request_url).send().await?;

        if !response.status().is_success() {
            return Err(AppError::tmdb_error(
                response.status().as_u16(),
                response.text().await?.as_str(),
            ));
        }

        let search_results = response.json::<movie::MovieSearch>().await?;

        Ok(search_results
            .results
            .into_iter()
            .map(Movie::from)
            .collect())
    }

    pub async fn popular(client: &TmdbClient) -> Result<Vec<Movie>, AppError> {
        let min_date = (chrono::Utc::now().date_naive() - chrono::Duration::days(30)).to_string();
        let max_date = (chrono::Utc::now().date_naive() + chrono::Duration::days(7)).to_string();

        let request_url = generate_endpoint(
            "discover/movie".to_string(),
            Some(HashMap::from([
                ("region", "AU"),
                ("include_video", "false"),
                ("page", "1"),
                ("sort_by", "popularity.desc"),
                ("with_release_type", "2|3"),
                ("release_date.gte", min_date.as_str()),
                ("release_date.lte", max_date.as_str()),
            ])),
        );

        let response = client.get(&request_url).send().await?;

        if !response.status().is_success() {
            return Err(AppError::tmdb_error(
                response.status().as_u16(),
                response.text().await?.as_str(),
            ));
        }

        let search_results = response.json::<movie::MovieSearch>().await?;

        Ok(search_results
            .results
            .into_iter()
            .map(Movie::from)
            .collect())
    }
}
