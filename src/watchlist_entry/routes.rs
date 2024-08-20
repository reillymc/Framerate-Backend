use crate::error_handler::CustomError;
use crate::movie::Movie;
use crate::user::placeholder_user;
use crate::watchlist_entry::NewWatchlistEntry;
use actix_web::delete;
use actix_web::{get, post, web, HttpResponse};
use serde::Deserialize;

use super::WatchlistEntry;

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SaveWatchlistEntryRequest {
    pub media_id: i32,
}

#[get("/watchlists/{media_type}/entries/{media_id}")]
async fn find_entry(path: web::Path<(String, i32)>) -> Result<HttpResponse, CustomError> {
    let user_id = placeholder_user();
    let (media_type, media_id) = path.into_inner();
    let entries = WatchlistEntry::find_entry(user_id, media_type, media_id)?;
    Ok(HttpResponse::Ok().json(entries))
}

#[get("/watchlists/{media_type}/entries")]
async fn find_all(media_type: web::Path<String>) -> Result<HttpResponse, CustomError> {
    let user_id = placeholder_user();
    let entries = WatchlistEntry::find_by_user_and_list(user_id, media_type.into_inner())?;
    Ok(HttpResponse::Ok().json(entries))
}

#[post("/watchlists/{media_type}/entries")]
async fn create(
    media_type: web::Path<String>,
    entry: web::Json<SaveWatchlistEntryRequest>,
) -> Result<HttpResponse, CustomError> {
    let movie = Movie::find(entry.media_id).await;

    let movie_details = match movie {
        Ok(movie) => movie,
        Err(_) => {
            return Err(CustomError::new(
                404,
                "The requested movie was not found".to_string(),
            ))
        }
    };

    let watchlist_entry_to_save = NewWatchlistEntry {
        user_id: placeholder_user(),
        media_id: entry.media_id,
        imdb_id: movie_details.imdb_id,
        media_title: movie_details.title,
        media_poster_uri: movie_details.poster_path,
        media_release_date: movie_details.release_date,
    };

    let watchlist = WatchlistEntry::create(media_type.into_inner(), watchlist_entry_to_save)?;
    Ok(HttpResponse::Ok().json(watchlist))
}

#[delete("/watchlists/{media_type}/entries/{media_id}")]
async fn update(path: web::Path<(String, i32)>) -> Result<HttpResponse, CustomError> {
    let (media_type, media_id) = path.into_inner();
    let watchlist = WatchlistEntry::delete(media_type, media_id)?;
    Ok(HttpResponse::Ok().json(watchlist))
}
