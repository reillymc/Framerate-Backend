use crate::error_handler::CustomError;
use crate::user::placeholder_user;
use actix_web::delete;
use actix_web::{get, post, web, HttpResponse};

use super::NewWatchlistEntry;
use super::WatchlistEntry;

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
    entry: web::Json<NewWatchlistEntry>,
) -> Result<HttpResponse, CustomError> {
    let watchlist = WatchlistEntry::create(media_type.into_inner(), entry.into_inner())?;
    Ok(HttpResponse::Ok().json(watchlist))
}

#[delete("/watchlists/{media_type}/entries/{media_id}")]
async fn update(path: web::Path<(String, i32)>) -> Result<HttpResponse, CustomError> {
    let (media_type, media_id) = path.into_inner();
    let watchlist = WatchlistEntry::delete(media_type, media_id)?;
    Ok(HttpResponse::Ok().json(watchlist))
}
