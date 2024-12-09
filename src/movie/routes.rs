use crate::{
    movie::Movie,
    tmdb::TmdbClient,
    utils::{
        jwt::Auth,
        response_body::{Error, Success},
    },
};
use actix_web::{get, web, HttpResponse, Responder};
use serde::Deserialize;

#[derive(Deserialize)]
struct SearchParameters {
    query: String,
}

#[get("/movies/search")]
async fn search(
    _: Auth,
    client: web::Data<TmdbClient>,
    params: web::Query<SearchParameters>,
) -> impl Responder {
    let movies = Movie::search(&client, &params.query).await;

    match movies {
        Ok(_) => HttpResponse::Ok().json(Success::new(movies.unwrap())),
        Err(err) => HttpResponse::InternalServerError().json(Error {
            message: err.to_string(),
        }),
    }
}

#[get("/movies/popular")]
async fn popular(_: Auth, client: web::Data<TmdbClient>) -> impl Responder {
    let movies = Movie::popular(&client).await;

    match movies {
        Ok(_) => HttpResponse::Ok().json(Success::new(movies.unwrap())),
        Err(err) => HttpResponse::InternalServerError().json(Error {
            message: err.to_string(),
        }),
    }
}

#[get("/movies/{movie_id}/details")]
async fn details(
    _: Auth,
    client: web::Data<TmdbClient>,
    movie_id: web::Path<i32>,
) -> impl Responder {
    let movie = Movie::find(&client, &movie_id.into_inner()).await;

    match movie {
        Ok(_) => HttpResponse::Ok().json(Success::new(movie.unwrap())),
        Err(err) => HttpResponse::InternalServerError().json(Error {
            message: err.to_string(),
        }),
    }
}
