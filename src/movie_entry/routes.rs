use super::MovieEntry;

use crate::db::DbPool;
use crate::error_handler::CustomError;
use crate::movie::Movie;
use crate::tmdb::TmdbClient;
use crate::utils::jwt::Auth;
use crate::utils::response_body::Success;
use crate::watchlist::Watchlist;
use actix_web::{delete, Responder};
use actix_web::{get, post, web};
use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SaveMovieEntryRequest {
    pub movie_id: i32,
}

#[derive(Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DeleteResponse {
    pub count: usize,
}

#[get("/movies/entries/{watchlist_id}/{movie_id}")]
async fn find(
    pool: web::Data<DbPool>,
    auth: Auth,
    path: web::Path<(String, i32)>,
) -> actix_web::Result<impl Responder> {
    let (_, movie_id) = path.into_inner();

    let movie_entry = web::block(move || {
        let mut conn = pool.get()?;
        let watchlist = Watchlist::find_default(&mut conn, auth.user_id, "movie")?;
        MovieEntry::find(&mut conn, auth.user_id, watchlist.watchlist_id, movie_id)
    })
    .await??;

    Ok(Success::new(movie_entry))
}

#[get("/movies/entries/{watchlist_id}")]
async fn find_all(
    pool: web::Data<DbPool>,
    auth: Auth,
    _: web::Path<String>,
) -> actix_web::Result<impl Responder> {
    let movie_entries = web::block(move || {
        let mut conn = pool.get()?;
        let watchlist = Watchlist::find_default(&mut conn, auth.user_id, "movie")?;
        MovieEntry::find_all(&mut conn, auth.user_id, watchlist.watchlist_id)
    })
    .await??;

    Ok(Success::new(movie_entries))
}

#[post("/movies/entries/{watchlist_id}")]
async fn create(
    pool: web::Data<DbPool>,
    client: web::Data<TmdbClient>,
    auth: Auth,
    _: web::Path<String>,
    watchlist_entry: web::Json<SaveMovieEntryRequest>,
) -> actix_web::Result<impl Responder> {
    let movie = Movie::find(&client, &watchlist_entry.movie_id).await?;

    let movie_entry = web::block(move || {
        let mut conn = pool.get()?;
        let watchlist = Watchlist::find_default(&mut conn, auth.user_id, "movie")?;

        let watchlist_entry_to_save = MovieEntry {
            watchlist_id: watchlist.watchlist_id,
            user_id: auth.user_id,
            movie_id: watchlist_entry.movie_id,
            imdb_id: movie.imdb_id,
            title: movie.title,
            poster_path: movie.poster_path,
            release_date: movie.release_date,
        };

        MovieEntry::create(&mut conn, watchlist_entry_to_save)
    })
    .await??;

    Ok(Success::new(movie_entry))
}

#[delete("/movies/entries/{watchlist_id}/{movie_id}")]
async fn delete(
    pool: web::Data<DbPool>,
    auth: Auth,
    path: web::Path<(String, i32)>,
) -> actix_web::Result<impl Responder> {
    let (_, movie_id) = path.into_inner();

    let count = web::block(move || {
        let mut conn = pool.get()?;
        let watchlist = Watchlist::find_default(&mut conn, auth.user_id, "movie")?;
        MovieEntry::delete(&mut conn, watchlist.watchlist_id, movie_id)
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
