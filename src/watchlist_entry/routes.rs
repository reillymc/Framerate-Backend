use super::WatchlistEntry;

use crate::utils::jwt::Auth;
use crate::utils::response_body::{Error, Success};
use crate::watchlist::Watchlist;
use actix_web::{delete, Responder};
use actix_web::{get, post, web, HttpResponse};
use serde::Deserialize;

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SaveWatchlistEntryRequest {
    pub media_id: i32,
}

#[get("/watchlists/{media_type}/entries/{media_id}")]
async fn find_entry(auth: Auth, path: web::Path<(String, i32)>) -> impl Responder {
    let (media_type, media_id) = path.into_inner();

    match WatchlistEntry::find_entry(auth.user_id, media_type, media_id) {
        Err(err) => HttpResponse::NotFound().json(Error {
            message: err.message,
        }),
        Ok(watchlist_entry) => HttpResponse::Ok().json(Success {
            data: watchlist_entry,
        }),
    }
}

#[get("/watchlists/{media_type}/entries")]
async fn find_all(auth: Auth, media_type: web::Path<String>) -> impl Responder {
    match WatchlistEntry::find_by_user_and_list(auth.user_id, media_type.into_inner()) {
        Err(err) => HttpResponse::InternalServerError().json(Error {
            message: err.message,
        }),
        Ok(entries) => HttpResponse::Ok().json(Success { data: entries }),
    }
}

#[post("/watchlists/{media_type}/entries")]
async fn create(
    auth: Auth,
    media_type: web::Path<String>,
    watchlist_entry: web::Json<SaveWatchlistEntryRequest>,
) -> impl Responder {
    let Ok(watchlist) = Watchlist::find_by_media_type(auth.user_id, media_type.clone()) else {
        return HttpResponse::NotFound().json(Error {
            message: "Watchlist not found".to_string(),
        });
    };

    let Ok(movie) = crate::movie::Movie::find(watchlist_entry.media_id).await else {
        return HttpResponse::NotFound().json(Error {
            message: "Movie not found".to_string(),
        });
    };

    let watchlist_entry_to_save = WatchlistEntry {
        watchlist_id: watchlist.watchlist_id,
        media_type: media_type.into_inner(),
        user_id: auth.user_id,
        media_id: watchlist_entry.media_id,
        imdb_id: movie.imdb_id,
        media_title: movie.title,
        media_poster_uri: movie.poster_path,
        media_release_date: movie.release_date,
    };

    let Ok(watchlist) = WatchlistEntry::create(watchlist_entry_to_save) else {
        return HttpResponse::InternalServerError().json(Error {
            message: "Watchlist entry could not be created".to_string(),
        });
    };

    HttpResponse::Ok().json(Success { data: watchlist })
}

#[delete("/watchlists/{media_type}/entries/{media_id}")]
async fn delete(auth: Auth, path: web::Path<(String, i32)>) -> impl Responder {
    let (media_type, media_id) = path.into_inner();

    let Ok(watchlist) = Watchlist::find_by_media_type(auth.user_id, media_type) else {
        return HttpResponse::NotFound().json(Error {
            message: "Watchlist not found".to_string(),
        });
    };

    let Ok(count) = WatchlistEntry::delete(watchlist.watchlist_id, media_id) else {
        return HttpResponse::NotFound().json(Error {
            message: "Watchlist entry not found".to_string(),
        });
    };

    if count == 0 {
        return HttpResponse::NotFound().json(Error {
            message: "Watchlist entry not found".to_string(),
        });
    }

    HttpResponse::Ok().json(Success { data: count })
}
