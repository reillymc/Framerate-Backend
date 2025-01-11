use crate::{
    movie::Movie,
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

#[utoipa::path(tag = "Movie", params(SearchParameters), responses((status = OK, body = Vec<Movie>)))]
#[get("/movies/search")]
async fn search(
    _: Auth,
    client: web::Data<TmdbClient>,
    params: web::Query<SearchParameters>,
) -> actix_web::Result<impl Responder> {
    let movies = Movie::search(&client, &params.query).await?;

    Ok(Success::new(movies))
}

#[utoipa::path(tag = "Movie", responses((status = OK, body = Vec<Movie>)))]
#[get("/movies/popular")]
async fn popular(_: Auth, client: web::Data<TmdbClient>) -> actix_web::Result<impl Responder> {
    let movies = Movie::popular(&client).await?;

    Ok(Success::new(movies))
}

#[utoipa::path(tag = "Movie", responses((status = OK, body = Movie),(status = NOT_FOUND)))]
#[get("/movies/{movie_id}/details")]
async fn details(
    _: Auth,
    client: web::Data<TmdbClient>,
    movie_id: web::Path<i32>,
) -> actix_web::Result<impl Responder> {
    let movie = Movie::find(&client, &movie_id.into_inner()).await?;

    Ok(Success::new(movie))
}
