use super::ShowEntry;

use crate::db::DbPool;
use crate::error_handler::CustomError;
use crate::show::Show;
use crate::tmdb::TmdbClient;
use crate::utils::jwt::Auth;
use crate::utils::response_body::Success;
use crate::watchlist::Watchlist;
use actix_web::{delete, Responder};
use actix_web::{get, post, web};
use chrono::Utc;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SaveShowEntryRequest {
    pub show_id: i32,
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DeleteResponse {
    pub count: usize,
}

#[get("/shows/entries/{watchlist_id}/{show_id}")]
async fn find(
    pool: web::Data<DbPool>,
    auth: Auth,
    path: web::Path<(String, i32)>,
) -> actix_web::Result<impl Responder> {
    let (_, show_id) = path.into_inner();

    let show_entry = web::block(move || {
        let mut conn = pool.get()?;
        let watchlist = Watchlist::find_default(&mut conn, auth.user_id, "show")?;
        ShowEntry::find(&mut conn, auth.user_id, watchlist.watchlist_id, show_id)
    })
    .await??;

    Ok(Success::new(show_entry))
}

#[get("/shows/entries/{watchlist_id}")]
async fn find_all(
    pool: web::Data<DbPool>,
    auth: Auth,
    _: web::Path<String>,
) -> actix_web::Result<impl Responder> {
    let show_entries = web::block(move || {
        let mut conn = pool.get()?;
        let watchlist = Watchlist::find_default(&mut conn, auth.user_id, "show")?;
        ShowEntry::find_all(&mut conn, auth.user_id, watchlist.watchlist_id)
    })
    .await??;

    Ok(Success::new(show_entries))
}

#[post("/shows/entries/{watchlist_id}")]
async fn create(
    pool: web::Data<DbPool>,
    client: web::Data<TmdbClient>,
    auth: Auth,
    _: web::Path<String>,
    watchlist_entry: web::Json<SaveShowEntryRequest>,
) -> actix_web::Result<impl Responder> {
    let show = Show::find(&client, &watchlist_entry.show_id).await?;

    let show_entry = web::block(move || {
        let mut conn = pool.get()?;
        let watchlist = Watchlist::find_default(&mut conn, auth.user_id, "show")?;

        let imdb_id = if let Some(external_ids) = show.external_ids {
            external_ids.imdb_id
        } else {
            None
        };

        let watchlist_entry_to_save = ShowEntry {
            watchlist_id: watchlist.watchlist_id,
            user_id: auth.user_id,
            show_id: watchlist_entry.show_id,
            imdb_id,
            name: show.name,
            poster_path: show.poster_path,
            first_air_date: show.first_air_date,
            last_air_date: show.last_air_date,
            next_air_date: show.next_air_date,
            status: show.status,
            updated_at: Utc::now().naive_utc().date(),
        };

        ShowEntry::create(&mut conn, watchlist_entry_to_save)
    })
    .await??;

    Ok(Success::new(show_entry))
}

#[delete("/shows/entries/{watchlist_id}/{show_id}")]
async fn delete(
    pool: web::Data<DbPool>,
    auth: Auth,
    path: web::Path<(String, i32)>,
) -> actix_web::Result<impl Responder> {
    let (_, show_id) = path.into_inner();

    let count = web::block(move || {
        let mut conn = pool.get()?;
        let watchlist = Watchlist::find_default(&mut conn, auth.user_id, "show")?;
        ShowEntry::delete(&mut conn, watchlist.watchlist_id, show_id)
    })
    .await??;

    if count == 0 {
        return Err(CustomError {
            status_code: 404,
            message: "Watchlist entry not found".to_string(),
        })?;
    }

    Ok(Success::new(DeleteResponse { count }))
}
