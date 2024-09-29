use crate::{
    show::Show,
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

#[get("/shows/search")]
async fn search(_: Auth, params: web::Query<SearchParameters>) -> impl Responder {
    let shows = Show::search(&params.query).await;

    match shows {
        Ok(_) => HttpResponse::Ok().json(Success {
            data: shows.unwrap(),
        }),
        Err(err) => HttpResponse::InternalServerError().json(Error {
            message: err.to_string(),
        }),
    }
}

#[get("/shows/popular")]
async fn popular(_: Auth) -> impl Responder {
    let shows = Show::popular().await;

    match shows {
        Ok(_) => HttpResponse::Ok().json(Success {
            data: shows.unwrap(),
        }),
        Err(err) => HttpResponse::InternalServerError().json(Error {
            message: err.to_string(),
        }),
    }
}

#[get("/shows/details/{show_id}")]
async fn find(_: Auth, show_id: web::Path<i32>) -> impl Responder {
    let show = Show::find(show_id.into_inner()).await;

    match show {
        Ok(_) => HttpResponse::Ok().json(Success {
            data: show.unwrap(),
        }),
        Err(err) => HttpResponse::InternalServerError().json(Error {
            message: err.to_string(),
        }),
    }
}
