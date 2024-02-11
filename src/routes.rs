use actix_web::web;

use crate::rating;
use crate::user;

pub fn init_routes(config: &mut web::ServiceConfig) {
    config.service(user::find);
    config.service(user::create);
    config.service(rating::find);
    config.service(rating::create);
}
