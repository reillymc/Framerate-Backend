use crate::error_handler::CustomError;
use crate::user::placeholder_user;
use actix_web::{get, post, put, web, HttpResponse};
use uuid::Uuid;

use super::NewWatchlist;
use super::Watchlist;

#[get("/watchlists/{media_type}")]
async fn find(media_type: web::Path<String>) -> Result<HttpResponse, CustomError> {
    // only allow media types of movie and show, else reutn error
    if media_type.as_str() != "movie" && media_type.as_str() != "show" {
        return Err(CustomError::new(400, "Invalid media type".to_string()));
    }

    let watchlist = Watchlist::find_by_media_type(media_type.into_inner())?;
    Ok(HttpResponse::Ok().json(watchlist))
}

#[get("/watchlists")]
async fn find_all() -> Result<HttpResponse, CustomError> {
    let user_id = placeholder_user();
    let watchlists = Watchlist::find_by_user(user_id)?;
    Ok(HttpResponse::Ok().json(watchlists))
}

#[post("/watchlists")]
async fn create(watchlist: web::Json<NewWatchlist>) -> Result<HttpResponse, CustomError> {
    let watchlist = Watchlist::create(watchlist.into_inner())?;
    Ok(HttpResponse::Ok().json(watchlist))
}

#[put("/watchlists/{watchlist_id}")]
async fn update(
    watchlist: web::Json<NewWatchlist>,
    watchlist_id: web::Path<Uuid>,
) -> Result<HttpResponse, CustomError> {
    let watchlist = Watchlist::update(watchlist_id.into_inner(), watchlist.into_inner())?;
    Ok(HttpResponse::Ok().json(watchlist))
}
