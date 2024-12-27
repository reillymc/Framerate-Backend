use super::NewWatchlist;
use super::Watchlist;

use crate::db::DbPool;
use crate::utils::{jwt::Auth, response_body::Success, AppError};
use actix_web::Responder;
use actix_web::{get, post, web};
use uuid::Uuid;

#[get("/watchlists/{media_type}")]
async fn find_by_media_type(
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

#[get("/watchlists")]
async fn find_all(pool: web::Data<DbPool>, auth: Auth) -> actix_web::Result<impl Responder> {
    let watchlists = web::block(move || {
        let mut conn = pool.get()?;
        Watchlist::find_by_user(&mut conn, auth.user_id)
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
    };

    let watchlist = web::block(move || {
        let mut conn = pool.get()?;
        Watchlist::create(&mut conn, watchlist)
    })
    .await??;

    Ok(Success::new(watchlist))
}
