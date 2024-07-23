use actix_web::web;

use crate::review;
use crate::user;

pub fn init_routes(config: &mut web::ServiceConfig) {
    config.service(user::find);
    config.service(user::create);
    config.service(review::find);
    config.service(review::find_all);
    config.service(review::find_by_media);
    config.service(review::create);
}
