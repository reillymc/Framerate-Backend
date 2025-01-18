use actix_web::get;
use actix_web::web;
use actix_web::HttpResponse;
use utoipa_actix_web::service_config::ServiceConfig;

use crate::administration;
use crate::authentication;
use crate::company;
use crate::movie;
use crate::movie_collection;
use crate::movie_review;
use crate::movie_watchlist;
use crate::season;
use crate::season_review;
use crate::show;
use crate::show_collection;
use crate::show_review;
use crate::show_watchlist;
use crate::user;

#[get("/health")]
async fn health() -> HttpResponse {
    let version = env!("CARGO_PKG_VERSION");
    HttpResponse::Ok()
        .append_header(("version", version))
        .finish()
}

pub fn init_routes(config: &mut ServiceConfig) {
    config
        .service(administration::generate_invite)
        .service(administration::generate_setup_token)
        .service(authentication::login)
        .service(authentication::register)
        .service(company::create)
        .service(company::delete)
        .service(company::find_all)
        .service(company::update)
        .service(movie_collection::create_entry)
        .service(movie_collection::create)
        .service(movie_collection::delete_entry)
        .service(movie_collection::delete)
        .service(movie_collection::find_all)
        .service(movie_collection::find_by_movie)
        .service(movie_collection::find)
        .service(movie_collection::update)
        .service(movie_review::create)
        .service(movie_review::find_all)
        .service(movie_review::find_by_movie_id)
        .service(movie_review::find_by_review_id)
        .service(movie_review::update)
        .service(movie_watchlist::create_entry)
        .service(movie_watchlist::delete_entry)
        .service(movie_watchlist::find_entry)
        .service(movie_watchlist::find)
        .service(movie::details)
        .service(movie::popular)
        .service(movie::search)
        .service(season_review::create)
        .service(season_review::find_by_review_id)
        .service(season_review::find_by_show_season)
        .service(season_review::update)
        .service(season::details)
        .service(show_collection::create_entry)
        .service(show_collection::create)
        .service(show_collection::delete_entry)
        .service(show_collection::delete)
        .service(show_collection::find_all)
        .service(show_collection::find_by_show)
        .service(show_collection::find)
        .service(show_collection::update)
        .service(show_review::create)
        .service(show_review::find_all)
        .service(show_review::find_by_review_id)
        .service(show_review::find_by_show_id)
        .service(show_review::update)
        .service(show_watchlist::create_entry)
        .service(show_watchlist::delete_entry)
        .service(show_watchlist::find_entry)
        .service(show_watchlist::find)
        .service(show::details)
        .service(show::popular)
        .service(show::search)
        .service(user::create)
        .service(user::find_all)
        .service(user::find)
        .service(user::update);
}

pub fn init_extra_routes(config: &mut web::ServiceConfig) {
    config.service(health);
}
