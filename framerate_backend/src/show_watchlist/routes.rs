use crate::collection::Collection;
use crate::db::DbPool;
use crate::show::{Show, SHOW_MEDIA_TYPE};
use crate::show_entry::ShowEntry;
use crate::tmdb::TmdbClient;
use crate::utils::response_body::{DeleteResponse, Success};
use crate::utils::{jwt::Auth, AppError};
use actix_web::{delete, Responder};
use actix_web::{get, post, web};
use chrono::{NaiveDate, Utc};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::Uuid;

const DEFAULT_WATCHLIST: &str = "watchlist";

#[derive(Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct ShowWatchlistEntry {
    pub show_id: i32,
    pub name: String,
    pub updated_at: NaiveDate,
    #[schema(nullable = false)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub imdb_id: Option<String>,
    #[schema(nullable = false)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub status: Option<String>,
    #[schema(nullable = false)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub poster_path: Option<String>,
    #[schema(nullable = false)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub first_air_date: Option<NaiveDate>,
    #[schema(nullable = false)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub last_air_date: Option<NaiveDate>,
    #[schema(nullable = false)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub next_air_date: Option<NaiveDate>,
}

impl From<ShowEntry> for ShowWatchlistEntry {
    fn from(value: ShowEntry) -> Self {
        ShowWatchlistEntry {
            first_air_date: value.first_air_date,
            imdb_id: value.imdb_id,
            last_air_date: value.last_air_date,
            name: value.name,
            next_air_date: value.next_air_date,
            poster_path: value.poster_path,
            show_id: value.show_id,
            status: value.status,
            updated_at: value.updated_at,
        }
    }
}

#[derive(Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct ShowWatchlist {
    pub name: String,
    #[schema(nullable = false)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub entries: Option<Vec<ShowWatchlistEntry>>,
}

impl ShowWatchlist {
    fn entries(mut self, entries: Vec<ShowEntry>) -> Self {
        self.entries = Some(entries.into_iter().map(ShowWatchlistEntry::from).collect());
        self
    }
}

impl From<Collection> for ShowWatchlist {
    fn from(value: Collection) -> Self {
        ShowWatchlist {
            name: value.name,
            entries: None,
        }
    }
}

#[derive(Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct SaveShowWatchlistEntryRequest {
    pub show_id: i32,
}

#[utoipa::path(tag = "Show Watchlist", responses((status = OK, body = ShowWatchlist)))]
#[get("/shows/watchlist")]
async fn find(pool: web::Data<DbPool>, auth: Auth) -> actix_web::Result<impl Responder> {
    let watchlist = web::block(move || {
        let mut conn = pool.get()?;
        let watchlist =
            Collection::find_default(&mut conn, auth.user_id, SHOW_MEDIA_TYPE, DEFAULT_WATCHLIST);

        let watchlist = match watchlist {
            Ok(watchlist) => Ok(watchlist),
            _ => Collection::create(
                &mut conn,
                Collection {
                    collection_id: Uuid::new_v4(),
                    media_type: SHOW_MEDIA_TYPE.to_string(),
                    user_id: auth.user_id,
                    name: "Show Watchlist".to_string(),
                    default_for: Some(DEFAULT_WATCHLIST.to_string()),
                },
            ),
        }?;

        let entries = ShowEntry::find_all(&mut conn, auth.user_id, watchlist.collection_id)?;

        Ok::<ShowWatchlist, AppError>(ShowWatchlist::from(watchlist).entries(entries))
    })
    .await??;

    Ok(Success::new(watchlist))
}

#[utoipa::path(tag = "Show Watchlist", responses((status = OK, body = ShowWatchlistEntry)))]
#[get("/shows/watchlist/{show_id}")]
async fn find_entry(
    pool: web::Data<DbPool>,
    auth: Auth,
    path: web::Path<i32>,
) -> actix_web::Result<impl Responder> {
    let show_id = path.into_inner();

    let show_entry = web::block(move || {
        let mut conn = pool.get()?;
        let collection =
            Collection::find_default(&mut conn, auth.user_id, SHOW_MEDIA_TYPE, DEFAULT_WATCHLIST)?;
        ShowEntry::find(&mut conn, auth.user_id, collection.collection_id, show_id)
    })
    .await??;

    Ok(Success::new(ShowWatchlistEntry::from(show_entry)))
}

#[utoipa::path(tag = "Show Watchlist", responses((status = OK, body = ShowWatchlistEntry)))]
#[post("/shows/watchlist")]
async fn create_entry(
    pool: web::Data<DbPool>,
    client: web::Data<TmdbClient>,
    auth: Auth,
    show_entry: web::Json<SaveShowWatchlistEntryRequest>,
) -> actix_web::Result<impl Responder> {
    let show = Show::find(&client, &show_entry.show_id).await?;

    let show_entry = web::block(move || {
        let mut conn = pool.get()?;
        let collection =
            Collection::find_default(&mut conn, auth.user_id, SHOW_MEDIA_TYPE, DEFAULT_WATCHLIST)?;

        let imdb_id = if let Some(external_ids) = show.external_ids {
            external_ids.imdb_id
        } else {
            None
        };

        let show_entry_to_save = ShowEntry {
            collection_id: collection.collection_id,
            user_id: auth.user_id,
            show_id: show_entry.show_id,
            imdb_id,
            name: show.name,
            poster_path: show.poster_path,
            first_air_date: show.first_air_date,
            last_air_date: show.last_air_date,
            next_air_date: show.next_air_date,
            status: show.status,
            updated_at: Utc::now().naive_utc().date(),
        };

        ShowEntry::create(&mut conn, show_entry_to_save)
    })
    .await??;

    Ok(Success::new(ShowWatchlistEntry::from(show_entry)))
}

#[utoipa::path(tag = "Show Watchlist", responses((status = OK, body = DeleteResponse)))]
#[delete("/shows/watchlist/{show_id}")]
async fn delete_entry(
    pool: web::Data<DbPool>,
    auth: Auth,
    path: web::Path<i32>,
) -> actix_web::Result<impl Responder> {
    let show_id = path.into_inner();

    let count = web::block(move || {
        let mut conn = pool.get()?;
        let collection =
            Collection::find_default(&mut conn, auth.user_id, SHOW_MEDIA_TYPE, DEFAULT_WATCHLIST)?;
        ShowEntry::delete(&mut conn, collection.collection_id, show_id)
    })
    .await??;

    if count == 0 {
        return Err(AppError::external(404, "Watchlist entry not found"))?;
    }

    Ok(Success::new(DeleteResponse { count }))
}
