use actix_web::web;

use crate::review;
use crate::review_company;
use crate::user;
use crate::watchlist;
use crate::watchlist_entry;

pub fn init_routes(config: &mut web::ServiceConfig) {
    config.service(user::find);
    config.service(user::find_all);
    config.service(user::create);
    config.service(user::update);
    config.service(review::find);
    config.service(review::find_all);
    config.service(review::find_by_media);
    config.service(review::create);
    config.service(review::update);
    config.service(watchlist_entry::create);
    config.service(watchlist_entry::find_entry);
    config.service(watchlist_entry::find_all);
    config.service(watchlist_entry::update);
    config.service(watchlist::find);
    config.service(watchlist::find_all);
    config.service(watchlist::create);
    config.service(watchlist::update);
    config.service(review_company::find_all);
    config.service(review_company::create);
    config.service(review_company::delete);
}
