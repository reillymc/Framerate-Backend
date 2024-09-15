use crate::{
    movie::Movie,
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
async fn search(_: Auth, params: web::Query<SearchParameters>) -> impl Responder {
    let movies = Movie::search(&params.query).await;

    match movies {
        Ok(_) => HttpResponse::Ok().json(Success {
            data: movies.unwrap(),
        }),
        Err(err) => HttpResponse::InternalServerError().json(Error {
            message: err.to_string(),
        }),
    }
}

#[get("/movies/popular")]
async fn popular(_: Auth) -> impl Responder {
    let movies = Movie::popular().await;

    match movies {
        Ok(_) => HttpResponse::Ok().json(Success {
            data: movies.unwrap(),
        }),
        Err(err) => HttpResponse::InternalServerError().json(Error {
            message: err.to_string(),
        }),
    }
}

#[get("/movies/details/{movie_id}")]
async fn find(_: Auth, movie_id: web::Path<i32>) -> impl Responder {
    let movie = Movie::find(movie_id.into_inner()).await;

    match movie {
        Ok(_) => HttpResponse::Ok().json(Success {
            data: movie.unwrap(),
        }),
        Err(err) => HttpResponse::InternalServerError().json(Error {
            message: err.to_string(),
        }),
    }
}
