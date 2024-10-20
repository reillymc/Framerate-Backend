use super::ShowEntry;

use crate::utils::jwt::Auth;
use crate::utils::response_body::{Error, Success};
use crate::watchlist::Watchlist;
use actix_web::{delete, Responder};
use actix_web::{get, post, web, HttpResponse};
use chrono::Utc;
use serde::{Deserialize, Serialize};

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SaveShowEntryRequest {
    pub show_id: i32,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DeleteResponse {
    pub count: usize,
}

#[get("/shows/entries/{watchlist_id}/{show_id}")]
async fn find(auth: Auth, path: web::Path<(String, i32)>) -> impl Responder {
    let (_, show_id) = path.into_inner();

    let Ok(watchlist) = Watchlist::find_default(auth.user_id, "show") else {
        return HttpResponse::NotFound().json(Error {
            message: "Watchlist not found".to_string(),
        });
    };

    match ShowEntry::find(auth.user_id, watchlist.watchlist_id, show_id) {
        Err(err) => HttpResponse::NotFound().json(Error {
            message: err.message,
        }),
        Ok(watchlist_entry) => HttpResponse::Ok().json(Success {
            data: watchlist_entry,
        }),
    }
}

#[get("/shows/entries/{watchlist_id}")]
async fn find_all(auth: Auth, _: web::Path<String>) -> impl Responder {
    let Ok(watchlist) = Watchlist::find_default(auth.user_id, "show") else {
        return HttpResponse::NotFound().json(Error {
            message: "Watchlist not found".to_string(),
        });
    };

    match ShowEntry::find_all(auth.user_id, watchlist.watchlist_id) {
        Err(err) => HttpResponse::InternalServerError().json(Error {
            message: err.message,
        }),
        Ok(entries) => HttpResponse::Ok().json(Success { data: entries }),
    }
}

#[post("/shows/entries/{watchlist_id}")]
async fn create(
    auth: Auth,
    _: web::Path<String>,
    watchlist_entry: web::Json<SaveShowEntryRequest>,
) -> impl Responder {
    let Ok(watchlist) = Watchlist::find_default(auth.user_id, "show") else {
        return HttpResponse::NotFound().json(Error {
            message: "Watchlist not found".to_string(),
        });
    };

    let Ok(show) = crate::show::Show::find(&watchlist_entry.show_id).await else {
        return HttpResponse::NotFound().json(Error {
            message: "Show not found".to_string(),
        });
    };

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

    let Ok(watchlist) = ShowEntry::create(watchlist_entry_to_save) else {
        return HttpResponse::InternalServerError().json(Error {
            message: "Watchlist entry could not be created".to_string(),
        });
    };

    HttpResponse::Ok().json(Success { data: watchlist })
}

#[delete("/shows/entries/{watchlist_id}/{show_id}")]
async fn delete(auth: Auth, path: web::Path<(String, i32)>) -> impl Responder {
    let (_, show_id) = path.into_inner();

    let Ok(watchlist) = Watchlist::find_default(auth.user_id, "show") else {
        return HttpResponse::NotFound().json(Error {
            message: "Watchlist not found".to_string(),
        });
    };

    let Ok(count) = ShowEntry::delete(watchlist.watchlist_id, show_id) else {
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
