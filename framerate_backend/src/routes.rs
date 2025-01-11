use actix_web::get;
use actix_web::web;
use actix_web::HttpResponse;
use utoipa_actix_web::service_config::ServiceConfig;

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
        .service(movie::details)
        .service(movie::popular)
        .service(movie::search);
}

pub fn init_undocumented_routes(config: &mut web::ServiceConfig) {
    config.service(health);

    config.service(authentication::login);
    config.service(authentication::setup);

    config.service(company::find_all);
    config.service(company::create);
    config.service(company::update);
    config.service(company::delete);

    config.service(movie_collection::find_all);
    config.service(movie_collection::find);
    config.service(movie_collection::create);
    config.service(movie_collection::update);
    config.service(movie_collection::delete);
    config.service(movie_collection::create_entry);
    config.service(movie_collection::delete_entry);
    config.service(movie_collection::find_by_movie);

    config.service(movie_review::find_by_review_id);
    config.service(movie_review::find_by_movie_id);
    config.service(movie_review::find_all);
    config.service(movie_review::create);
    config.service(movie_review::update);

    config.service(movie_watchlist::find);
    config.service(movie_watchlist::find_entry);
    config.service(movie_watchlist::create_entry);
    config.service(movie_watchlist::delete_entry);

    config.service(season::details);

    config.service(season_review::find_by_show_season);
    config.service(season_review::find_by_review_id);
    config.service(season_review::create);
    config.service(season_review::update);

    config.service(show::details);
    config.service(show::popular);
    config.service(show::search);

    config.service(show_collection::find_all);
    config.service(show_collection::find);
    config.service(show_collection::create);
    config.service(show_collection::update);
    config.service(show_collection::delete);
    config.service(show_collection::create_entry);
    config.service(show_collection::delete_entry);
    config.service(show_collection::find_by_show);

    config.service(show_review::find_by_review_id);
    config.service(show_review::find_by_show_id);
    config.service(show_review::find_all);
    config.service(show_review::create);
    config.service(show_review::update);

    config.service(show_watchlist::find);
    config.service(show_watchlist::find_entry);
    config.service(show_watchlist::create_entry);
    config.service(show_watchlist::delete_entry);

    config.service(user::find);
    config.service(user::find_all);
    config.service(user::create);
    config.service(user::update);
}
