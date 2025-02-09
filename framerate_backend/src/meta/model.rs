use diesel::prelude::*;
use serde::{Deserialize, Serialize};
use serde_json::Map;
use utoipa::ToSchema;

use crate::{db::DbConnection, schema::server_meta, utils::AppError};

pub enum MetaEntryKey {
    ClientConfig,
}

#[derive(Serialize, ToSchema)]
#[serde(untagged)]
pub enum MetaEntry {
    ClientConfig(ClientConfig),
}

#[derive(Deserialize, Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct LinkRoutes {
    #[schema(nullable = false)]
    #[serde(skip_serializing_if = "Option::is_none")]
    movie: Option<String>,
    #[schema(nullable = false)]
    #[serde(skip_serializing_if = "Option::is_none")]
    show: Option<String>,
    #[schema(nullable = false)]
    #[serde(skip_serializing_if = "Option::is_none")]
    season: Option<String>,
    #[schema(nullable = false)]
    #[serde(skip_serializing_if = "Option::is_none")]
    episode: Option<String>,
}

#[derive(Deserialize, Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct LinkIcon {
    uri: String,
    #[schema(nullable = false)]
    #[serde(skip_serializing_if = "Option::is_none")]
    uri_dark: Option<String>,
}

#[derive(Deserialize, Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct MediaExternalLink {
    name: String,
    #[serde(default)]
    enabled: bool,
    links: LinkRoutes,
    icon: LinkIcon,
}

fn default_media_external_links() -> Vec<MediaExternalLink> {
    Vec::from([
        MediaExternalLink {
            name: "TMDB".to_string(),
            enabled: true,
            icon: LinkIcon {
                uri: "https://www.themoviedb.org/assets/2/v4/logos/v2/blue_square_1-5bdc75aaebeb75dc7ae79426ddd9be3b2be1e342510f8202baf6bffa71d7f5c4.svg".to_string(),
                uri_dark: None
            },
            links: LinkRoutes {
                movie: Some("https://www.themoviedb.org/movie/{{tmdbId}}".to_string()),
                show: Some("https://www.themoviedb.org/tv/{{tmdbId}}".to_string()),
                season: Some("https://www.themoviedb.org/tv/{{tmdbId}}/season/{{seasonNumber}}".to_string()),
                episode: Some("https://www.themoviedb.org/tv/{{tmdbId}}/season/{{seasonNumber}}/episode/{{episodeNumber}}".to_string()),
            }
        },
        MediaExternalLink {
            name: "IMDB".to_string(),
            enabled: true,
            icon: LinkIcon {
                uri: "https://upload.wikimedia.org/wikipedia/commons/6/69/IMDB_Logo_2016.svg".to_string(),
                uri_dark: None
            },
            links: LinkRoutes {
                movie: Some("https://www.imdb.com/title/{{imdbId}}".to_string()),
                show: Some("https://www.imdb.com/title/{{imdbId}}".to_string()),
                season: Some("https://www.imdb.com/title/{{imdbId}}/episodes?season={{seasonNumber}}".to_string()),
                episode: None
            }
        },
    ])
}

#[derive(Deserialize, Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct ClientConfig {
    #[serde(default = "default_media_external_links")]
    pub media_external_links: Vec<MediaExternalLink>,
}

impl From<MetaEntryKey> for String {
    fn from(value: MetaEntryKey) -> Self {
        match value {
            MetaEntryKey::ClientConfig => "client_config".to_string(),
        }
    }
}

impl TryFrom<MetaEntry> for MetaEntryInternal {
    type Error = AppError;

    fn try_from(value: MetaEntry) -> Result<Self, AppError> {
        let (key, value) = match value {
            MetaEntry::ClientConfig(value) => (MetaEntryKey::ClientConfig, value),
        };

        let value = serde_json::to_value(&value)
            .map_err(|_| AppError::external(500, "unable to serialise config"))?;

        Ok(MetaEntryInternal {
            key: key.into(),
            value,
        })
    }
}

impl TryFrom<MetaEntryInternal> for MetaEntry {
    type Error = AppError;

    fn try_from(value: MetaEntryInternal) -> Result<Self, AppError> {
        let value = match value.key.as_str() {
            "client_config" => {
                let value = ClientConfig::deserialize(value.value)
                    .map_err(|_| AppError::external(500, "unable to parse config"))?;
                Ok(MetaEntry::ClientConfig(value))
            }
            _ => Err(AppError::external(404, "entry not found")),
        }?;

        Ok(value)
    }
}

#[derive(Selectable, Queryable, AsChangeset, Insertable)]
#[diesel(table_name = server_meta)]
struct MetaEntryInternal {
    pub key: String,
    pub value: serde_json::Value,
}

impl MetaEntry {
    pub fn find(conn: &mut DbConnection, key: MetaEntryKey) -> Result<Self, AppError> {
        let key = String::from(key);
        let entry = server_meta::table
            .select(MetaEntryInternal::as_select())
            .filter(server_meta::key.eq(&key))
            .first(conn);

        if let Ok(entry) = entry {
            return MetaEntry::try_from(entry);
        }

        return MetaEntry::try_from(MetaEntryInternal {
            key,
            value: serde_json::Value::Object(Map::<String, serde_json::Value>::new()),
        });
    }

    pub fn update(conn: &mut DbConnection, entry: MetaEntry) -> Result<Self, AppError> {
        let entry = MetaEntryInternal::try_from(entry)?;
        let updated_entry: MetaEntryInternal = diesel::insert_into(server_meta::table)
            .values(&entry)
            .on_conflict(server_meta::key)
            .do_update()
            .set(&entry)
            .get_result(conn)?;

        MetaEntry::try_from(updated_entry)
    }
}
