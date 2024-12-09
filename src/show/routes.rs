use crate::{
    show::Show,
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

#[get("/shows/search")]
async fn search(
    _: Auth,
    client: web::Data<TmdbClient>,
    params: web::Query<SearchParameters>,
) -> impl Responder {
    let shows = Show::search(&client, &params.query).await;

    match shows {
        Ok(_) => HttpResponse::Ok().json(Success::new(shows.unwrap())),
        Err(err) => HttpResponse::InternalServerError().json(Error {
            message: err.to_string(),
        }),
    }
}

#[get("/shows/popular")]
async fn popular(_: Auth, client: web::Data<TmdbClient>) -> impl Responder {
    let shows = Show::popular(&client).await;

    match shows {
        Ok(_) => HttpResponse::Ok().json(Success::new(shows.unwrap())),
        Err(err) => HttpResponse::InternalServerError().json(Error {
            message: err.to_string(),
        }),
    }
}

#[get("/shows/{show_id}/details")]
async fn details(
    _: Auth,
    client: web::Data<TmdbClient>,
    show_id: web::Path<i32>,
) -> impl Responder {
    let show = Show::find(&client, &show_id.into_inner()).await;

    match show {
        Ok(_) => HttpResponse::Ok().json(Success::new(show.unwrap())),
        Err(err) => HttpResponse::InternalServerError().json(Error {
            message: err.to_string(),
        }),
    }
}
