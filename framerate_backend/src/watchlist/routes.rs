use super::NewWatchlist;
use super::Watchlist;

use crate::db::DbPool;
use crate::utils::{jwt::Auth, response_body::Success, AppError};
use actix_web::Responder;
use actix_web::{get, post, web};
use uuid::Uuid;

#[get("/watchlists/{watchlist_id}")]
async fn find(
    pool: web::Data<DbPool>,
    auth: Auth,
    watchlist_id: web::Path<Uuid>,
) -> actix_web::Result<impl Responder> {
    let watchlist = web::block(move || {
        let mut conn = pool.get()?;
        Watchlist::find(&mut conn, auth.user_id, &watchlist_id)
    })
    .await??;

    Ok(Success::new(watchlist))
}

#[get("/watchlists/type/{media_type}/default")]
async fn find_default(
    pool: web::Data<DbPool>,
    auth: Auth,
    media_type: web::Path<String>,
) -> actix_web::Result<impl Responder> {
    if media_type.as_str() != "movie" && media_type.as_str() != "show" {
        Err(AppError::external(400, "Invalid media type"))?
    }

    let watchlist = web::block(move || {
        let mut conn = pool.get()?;
        Watchlist::find_default(&mut conn, auth.user_id, &media_type)
    })
    .await??;

    Ok(Success::new(watchlist))
}

#[get("/watchlists/type/{media_type}")]
async fn find_all(
    pool: web::Data<DbPool>,
    auth: Auth,
    media_type: web::Path<String>,
) -> actix_web::Result<impl Responder> {
    let watchlists = web::block(move || {
        let mut conn = pool.get()?;
        Watchlist::find_by_media_type(&mut conn, auth.user_id, &media_type)
    })
    .await??;

    Ok(Success::new(watchlists))
}

#[post("/watchlists")]
async fn create(
    pool: web::Data<DbPool>,
    auth: Auth,
    params: web::Json<NewWatchlist>,
) -> actix_web::Result<impl Responder> {
    let params = params.into_inner();

    let watchlist = Watchlist {
        watchlist_id: Uuid::new_v4(),
        user_id: auth.user_id,
        name: params.name,
        media_type: params.media_type,
        default_for: None,
    };

    let watchlist = web::block(move || {
        let mut conn = pool.get()?;
        Watchlist::create(&mut conn, watchlist)
    })
    .await??;

    Ok(Success::new(watchlist))
}
