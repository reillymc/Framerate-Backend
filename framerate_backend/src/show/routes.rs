use crate::{
    show::Show,
    tmdb::TmdbClient,
    utils::{jwt::Auth, response_body::Success},
};
use actix_web::{get, web, Responder};
use serde::Deserialize;
use utoipa::IntoParams;

#[derive(Deserialize, IntoParams)]
struct SearchParameters {
    query: String,
}

#[utoipa::path(tag = "Show", params(SearchParameters), responses((status = OK, body = Vec<Show>)))]
#[get("/shows/search")]
async fn search(
    _: Auth,
    client: web::Data<TmdbClient>,
    params: web::Query<SearchParameters>,
) -> actix_web::Result<impl Responder> {
    let shows = Show::search(&client, &params.query).await?;

    Ok(Success::new(shows))
}

#[utoipa::path(tag = "Show", responses((status = OK, body = Vec<Show>)))]
#[get("/shows/popular")]
async fn popular(_: Auth, client: web::Data<TmdbClient>) -> actix_web::Result<impl Responder> {
    let shows = Show::popular(&client).await?;

    Ok(Success::new(shows))
}

#[utoipa::path(tag = "Show", responses((status = OK, body = Show),(status = NOT_FOUND)))]
#[get("/shows/{show_id}/details")]
async fn details(
    _: Auth,
    client: web::Data<TmdbClient>,
    show_id: web::Path<i32>,
) -> actix_web::Result<impl Responder> {
    let show = Show::find(&client, &show_id.into_inner()).await?;

    Ok(Success::new(show))
}
