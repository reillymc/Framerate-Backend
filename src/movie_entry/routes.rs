use super::MovieEntry;

use crate::utils::jwt::Auth;
use crate::utils::response_body::{Error, Success};
use crate::watchlist::Watchlist;
use actix_web::{delete, Responder};
use actix_web::{get, post, web, HttpResponse};
use serde::{Deserialize, Serialize};

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SaveMovieEntryRequest {
    pub movie_id: i32,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DeleteResponse {
    pub count: usize,
}

#[get("/movies/entries/{watchlist_id}/{movie_id}")]
async fn find(auth: Auth, path: web::Path<(String, i32)>) -> impl Responder {
    let (_, movie_id) = path.into_inner();

    let Ok(watchlist) = Watchlist::find_default(auth.user_id, "movie") else {
        return HttpResponse::NotFound().json(Error {
            message: "Watchlist not found".to_string(),
        });
    };

    match MovieEntry::find(auth.user_id, watchlist.watchlist_id, movie_id) {
        Err(err) => HttpResponse::NotFound().json(Error {
            message: err.message,
        }),
        Ok(watchlist_entry) => HttpResponse::Ok().json(Success {
            data: watchlist_entry,
        }),
    }
}

#[get("/movies/entries/{watchlist_id}")]
async fn find_all(auth: Auth, _: web::Path<String>) -> impl Responder {
    let Ok(watchlist) = Watchlist::find_default(auth.user_id, "movie") else {
        return HttpResponse::NotFound().json(Error {
            message: "Watchlist not found".to_string(),
        });
    };

    match MovieEntry::find_all(auth.user_id, watchlist.watchlist_id) {
        Err(err) => HttpResponse::InternalServerError().json(Error {
            message: err.message,
        }),
        Ok(entries) => HttpResponse::Ok().json(Success { data: entries }),
    }
}

#[post("/movies/entries/{watchlist_id}")]
async fn create(
    auth: Auth,
    _: web::Path<String>,
    watchlist_entry: web::Json<SaveMovieEntryRequest>,
) -> impl Responder {
    let Ok(watchlist) = Watchlist::find_default(auth.user_id, "movie") else {
        return HttpResponse::NotFound().json(Error {
            message: "Watchlist not found".to_string(),
        });
    };

    let Ok(movie) = crate::movie::Movie::find(&watchlist_entry.movie_id).await else {
        return HttpResponse::NotFound().json(Error {
            message: "Movie not found".to_string(),
        });
    };

    let watchlist_entry_to_save = MovieEntry {
        watchlist_id: watchlist.watchlist_id,
        user_id: auth.user_id,
        movie_id: watchlist_entry.movie_id,
        imdb_id: movie.imdb_id,
        title: movie.title,
        poster_path: movie.poster_path,
        release_date: movie.release_date,
    };

    let Ok(watchlist) = MovieEntry::create(watchlist_entry_to_save) else {
        return HttpResponse::InternalServerError().json(Error {
            message: "Watchlist entry could not be created".to_string(),
        });
    };

    HttpResponse::Ok().json(Success { data: watchlist })
}

#[delete("/movies/entries/{watchlist_id}/{movie_id}")]
async fn delete(auth: Auth, path: web::Path<(String, i32)>) -> impl Responder {
    let (_, movie_id) = path.into_inner();

    let Ok(watchlist) = Watchlist::find_default(auth.user_id, "movie") else {
        return HttpResponse::NotFound().json(Error {
            message: "Watchlist not found".to_string(),
        });
    };

    let Ok(count) = MovieEntry::delete(watchlist.watchlist_id, movie_id) else {
        return HttpResponse::NotFound().json(Error {
            message: "Watchlist entry not found".to_string(),
        });
    };

    if count == 0 {
        return HttpResponse::NotFound().json(Error {
            message: "Watchlist entry not found".to_string(),
        });
    }

    HttpResponse::Ok().json(Success {
        data: DeleteResponse { count },
    })
}
