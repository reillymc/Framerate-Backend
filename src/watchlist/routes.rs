use super::NewWatchlist;
use super::Watchlist;

use crate::utils::jwt::Auth;
use crate::utils::response_body::{Error, Success};
use actix_web::Responder;
use actix_web::{get, post, web, HttpResponse};
use uuid::Uuid;

#[get("/watchlists/{media_type}")]
async fn find(auth: Auth, media_type: web::Path<String>) -> impl Responder {
    if media_type.as_str() != "movie" && media_type.as_str() != "show" {
        return HttpResponse::BadRequest().json(Error {
            message: "Invalid media type".to_string(),
        });
    }

    match Watchlist::find_default(auth.user_id, &media_type) {
        Err(err) => HttpResponse::InternalServerError().json(Error {
            message: err.message,
        }),
        Ok(watchlist) => HttpResponse::Ok().json(Success { data: watchlist }),
    }
}

#[get("/watchlists")]
async fn find_all(auth: Auth) -> impl Responder {
    match Watchlist::find_by_user(auth.user_id) {
        Err(err) => HttpResponse::InternalServerError().json(Error {
            message: err.message,
        }),
        Ok(watchlists) => HttpResponse::Ok().json(Success { data: watchlists }),
    }
}

#[post("/watchlists")]
async fn create(auth: Auth, params: web::Json<NewWatchlist>) -> impl Responder {
    let watchlist = Watchlist {
        watchlist_id: Uuid::new_v4(),
        user_id: auth.user_id,
        name: params.name.clone(),
        media_type: params.media_type.clone(),
    };

    let watchlist = Watchlist::create(watchlist);

    match watchlist {
        Err(err) => HttpResponse::InternalServerError().json(Error {
            message: err.message,
        }),
        Ok(watchlist) => HttpResponse::Ok().json(Success { data: watchlist }),
    }
}
