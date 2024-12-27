use actix_web::get;
use actix_web::web;
use actix_web::HttpResponse;

use crate::authentication;
use crate::company;
use crate::movie;
use crate::movie_entry;
use crate::movie_review;
use crate::season;
use crate::season_review;
use crate::show;
use crate::show_entry;
use crate::show_review;
use crate::user;
use crate::watchlist;

#[get("/health")]
async fn health() -> HttpResponse {
    let version = env!("CARGO_PKG_VERSION");
    HttpResponse::Ok()
        .append_header(("version", version))
        .finish()
}

pub fn init_routes(config: &mut web::ServiceConfig) {
    config.service(health);

    config.service(authentication::login);
    config.service(authentication::setup);

    config.service(company::find_all);
    config.service(company::create);
    config.service(company::update);
    config.service(company::delete);

    config.service(user::find);
    config.service(user::find_all);
    config.service(user::create);
    config.service(user::update);

    config.service(watchlist::find_by_media_type);
    config.service(watchlist::find_all);
    config.service(watchlist::create);

    config.service(movie::details);
    config.service(movie::popular);
    config.service(movie::search);

    config.service(movie_entry::find);
    config.service(movie_entry::find_all);
    config.service(movie_entry::create);
    config.service(movie_entry::delete);

    config.service(movie_review::find_by_review_id);
    config.service(movie_review::find_by_movie_id);
    config.service(movie_review::find_all);
    config.service(movie_review::create);
    config.service(movie_review::update);

    config.service(show::details);
    config.service(show::popular);
    config.service(show::search);

    config.service(show_entry::find);
    config.service(show_entry::find_all);
    config.service(show_entry::create);
    config.service(show_entry::delete);

    config.service(show_review::find_by_review_id);
    config.service(show_review::find_by_show_id);
    config.service(show_review::find_all);
    config.service(show_review::create);
    config.service(show_review::update);

    config.service(season::details);

    config.service(season_review::find_by_show_season);
    config.service(season_review::find_by_review_id);
    config.service(season_review::create);
    config.service(season_review::update);
}
