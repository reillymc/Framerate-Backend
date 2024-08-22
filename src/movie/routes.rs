use crate::{error_handler::CustomError, movie::Movie};
use actix_web::{get, web, HttpResponse};
use serde::Deserialize;

#[derive(Deserialize)]
struct SearchParameters {
    query: String,
}

#[get("/movies/search")]
async fn search(params: web::Query<SearchParameters>) -> Result<HttpResponse, CustomError> {
    let movies = Movie::search(&params.query).await;

    match movies {
        Ok(_) => Ok(HttpResponse::Ok().json(movies.unwrap())),
        Err(err) => Err(err),
    }
}

#[get("/movies/popular")]
async fn popular() -> Result<HttpResponse, CustomError> {
    let movies = Movie::popular().await;

    match movies {
        Ok(_) => Ok(HttpResponse::Ok().json(movies.unwrap())),
        Err(err) => Err(err),
    }
}

#[get("/movies/details/{movie_id}")]
async fn find(movie_id: web::Path<i32>) -> Result<HttpResponse, CustomError> {
    let movie = Movie::find(movie_id.into_inner()).await;

    match movie {
        Ok(_) => Ok(HttpResponse::Ok().json(movie.unwrap())),
        Err(err) => Err(err),
    }
}
