use actix_web::web;

use crate::authentication;
use crate::movie;
use crate::movie_review;
use crate::season;
use crate::season_review;
use crate::show;
use crate::show_review;
use crate::user;
use crate::watchlist;
use crate::watchlist_entry;

pub fn init_routes(config: &mut web::ServiceConfig) {
    config.service(authentication::login);
    config.service(authentication::setup);

    config.service(user::find);
    config.service(user::find_all);
    config.service(user::create);
    config.service(user::update);

    config.service(watchlist::find);
    config.service(watchlist::find_all);
    config.service(watchlist::create);

    config.service(watchlist_entry::find_entry);
    config.service(watchlist_entry::find_all);
    config.service(watchlist_entry::create);
    config.service(watchlist_entry::delete);

    config.service(movie::find);
    config.service(movie::popular);
    config.service(movie::search);

    config.service(movie_review::find_by_review_id);
    config.service(movie_review::find_all);
    config.service(movie_review::find_by_movie_id);
    config.service(movie_review::create);
    config.service(movie_review::update);

    config.service(show::find);
    config.service(show::popular);
    config.service(show::search);

    config.service(show_review::find_by_review_id);
    config.service(show_review::find_all);
    config.service(show_review::find_by_show_id);
    config.service(show_review::create);
    config.service(show_review::update);

    config.service(season::find);

    config.service(season_review::find_by_show_season);
    config.service(season_review::find_by_review_id);
    config.service(season_review::create);
    config.service(season_review::update);
}
