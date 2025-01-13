use crate::{
    season::Season,
    tmdb::{generate_endpoint, TmdbClient},
    utils::AppError,
};
use chrono::NaiveDate;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tmdb_api::{show, utils::serialization::empty_string_as_none};
use utoipa::ToSchema;

pub const SHOW_MEDIA_TYPE: &str = "show";

#[derive(ToSchema, Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Role {
    #[schema(nullable = false)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub character: Option<String>,
    pub episode_count: i64,
}

#[derive(ToSchema, Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Job {
    pub job: String,
    pub episode_count: i64,
}

#[derive(ToSchema, Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct ShowCast {
    pub id: i64,
    #[schema(nullable = false)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub known_for_department: Option<String>,
    #[schema(nullable = false)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    pub popularity: f64,
    #[schema(nullable = false)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub profile_path: Option<String>,
    pub roles: Vec<Role>,
    pub total_episode_count: i64,
}

#[derive(ToSchema, Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct ShowCrew {
    pub id: i64,
    #[schema(nullable = false)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub known_for_department: Option<String>,
    #[schema(nullable = false)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    pub popularity: f64,
    #[schema(nullable = false)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub profile_path: Option<String>,
    #[schema(nullable = false)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub department: Option<String>,
    pub jobs: Vec<Job>,
    pub total_episode_count: i64,
}

#[derive(ToSchema, Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct ShowCredits {
    pub cast: Vec<ShowCast>,
    pub crew: Vec<ShowCrew>,
}

impl From<show::Crew> for ShowCrew {
    fn from(crew: show::Crew) -> Self {
        ShowCrew {
            id: crew.id,
            known_for_department: crew.known_for_department,
            name: crew.name,
            popularity: crew.popularity,
            profile_path: crew.profile_path,
            department: crew.department,
            total_episode_count: crew.total_episode_count,
            jobs: crew.jobs.into_iter().map(Job::from).collect(),
        }
    }
}
impl From<show::Cast> for ShowCast {
    fn from(cast: show::Cast) -> Self {
        ShowCast {
            id: cast.id,
            known_for_department: cast.known_for_department,
            name: cast.name,
            popularity: cast.popularity,
            profile_path: cast.profile_path,
            total_episode_count: cast.total_episode_count,
            roles: cast.roles.into_iter().map(Role::from).collect(),
        }
    }
}

impl From<show::Job> for Job {
    fn from(job: show::Job) -> Self {
        Job {
            episode_count: job.episode_count,
            job: job.job,
        }
    }
}

impl From<show::Role> for Role {
    fn from(role: show::Role) -> Self {
        Role {
            episode_count: role.episode_count,
            character: role.character,
        }
    }
}

impl From<show::Credits> for ShowCredits {
    fn from(credits: show::Credits) -> Self {
        let mut cast = credits.cast;
        cast.sort_by(|a, b| a.order.cmp(&b.order));
        let cast = cast.into_iter().take(20).map(ShowCast::from).collect();

        let crew = credits
            .crew
            .into_iter()
            .take(20)
            .map(ShowCrew::from)
            .collect();

        ShowCredits { cast, crew }
    }
}

#[derive(ToSchema, Debug, Deserialize, Serialize)]
#[serde(rename_all(serialize = "camelCase"))]
pub struct ExternalIds {
    #[schema(nullable = false)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub imdb_id: Option<String>,
    #[schema(nullable = false)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tvdb_id: Option<i64>,
}

#[derive(ToSchema, Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Show {
    pub id: i32,
    pub name: String,
    #[schema(nullable = false)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub poster_path: Option<String>,
    #[schema(nullable = false)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub backdrop_path: Option<String>,
    #[schema(nullable = false)]
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(default, deserialize_with = "empty_string_as_none")]
    pub first_air_date: Option<NaiveDate>,
    #[schema(nullable = false)]
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(default, deserialize_with = "empty_string_as_none")]
    pub last_air_date: Option<NaiveDate>,
    #[schema(nullable = false)]
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(default, deserialize_with = "empty_string_as_none")]
    pub next_air_date: Option<NaiveDate>,
    #[schema(nullable = false)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub status: Option<String>,
    #[schema(nullable = false)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub overview: Option<String>,
    #[schema(nullable = false)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tagline: Option<String>,
    #[schema(nullable = false)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub popularity: Option<f32>,
    #[schema(nullable = false)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub external_ids: Option<ExternalIds>,
    #[schema(nullable = false)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub seasons: Option<Vec<Season>>,
    #[schema(nullable = false)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub credits: Option<ShowCredits>,
}

pub const SHOW_ACTIVE_STATUSES: [&str; 4] =
    ["Returning Series", "Planned", "In Production", "Pilot"];

impl From<show::ShowSearch> for Show {
    fn from(show: show::ShowSearch) -> Self {
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
            credits: None,
        }
    }
}

impl From<show::Show> for Show {
    fn from(show: show::Show) -> Self {
        let seasons = show.seasons.map(|seasons| {
            seasons
                .into_iter()
                .map(|season| Season {
                    show_id: show.id,
                    season_number: season.season_number,
                    name: season.name,
                    overview: season.overview,
                    poster_path: season.poster_path,
                    air_date: season.air_date,
                    episode_count: season.episode_count,
                    episodes: None,
                })
                .collect()
        });

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

        let external_ids = if let Some(external_ids) = show.external_ids {
            Some(ExternalIds {
                imdb_id: external_ids.imdb_id,
                tvdb_id: external_ids.tvdb_id,
            })
        } else {
            None
        };

        let credits = if let Some(credits) = show.aggregate_credits {
            Some(ShowCredits::from(credits))
        } else {
            None
        };

        Show {
            id: show.id,
            name: show.name,
            overview: show.overview,
            tagline: show.tagline,
            popularity: show.popularity,
            external_ids,
            backdrop_path: show.backdrop_path,
            first_air_date: show.first_air_date,
            last_air_date,
            next_air_date,
            status: show.status,
            poster_path: show.poster_path,
            seasons,
            credits,
        }
    }
}

#[derive(Deserialize, Debug)]
struct ShowSearchResults {
    pub results: Vec<show::ShowSearch>,
}

impl Show {
    pub async fn find(client: &TmdbClient, id: &i32) -> Result<Show, AppError> {
        let generate_endpoint = generate_endpoint(
            format!("tv/{id}"),
            Some(HashMap::from([(
                "append_to_response",
                "external_ids,aggregate_credits",
            )])),
        );
        let request_url = generate_endpoint;

        let response = client.get(&request_url).send().await?;

        if !response.status().is_success() {
            return Err(AppError::tmdb_error(
                response.status().as_u16(),
                response.text().await?.as_str(),
            ));
        }

        let show = response.json::<show::Show>().await?;

        Ok(Show::from(show))
    }

    pub async fn search(client: &TmdbClient, query: &str) -> Result<Vec<Show>, AppError> {
        let request_url = generate_endpoint(
            "search/tv".to_string(),
            Some(HashMap::from([("query", query), ("page", "1")])),
        );

        let response = client.get(&request_url).send().await?;

        if !response.status().is_success() {
            return Err(AppError::tmdb_error(
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

    pub async fn popular(client: &TmdbClient) -> Result<Vec<Show>, AppError> {
        let max_date = (chrono::Utc::now().date_naive() + chrono::Duration::weeks(26)).to_string();

        let request_url = generate_endpoint(
            "discover/tv".to_string(),
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
            return Err(AppError::tmdb_error(
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
