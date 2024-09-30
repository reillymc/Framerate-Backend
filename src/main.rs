extern crate diesel;
// #[macro_use]
extern crate diesel_migrations;

use actix_cors::Cors;
use actix_web::{middleware::Logger, App, HttpServer};
use db::establish_connection;
use env_logger::Env;
use std::env;

mod authentication;
mod db;
mod error_handler;
mod movie;
mod review;
mod review_company;
mod routes;
mod schema;
mod show;
mod user;
mod utils;
mod watchlist;
mod watchlist_entry;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let connection = &mut establish_connection();
    db::run_db_migrations(connection);

    env_logger::init_from_env(Env::default().default_filter_or("info"));

    let host = env::var("HOST").unwrap_or("localhost".to_string());
    let port = env::var("PORT").unwrap_or("3000".to_string());
    println!("Server starting at http://{host}:{port}");

    HttpServer::new(|| {
        App::new()
            .wrap(Cors::default())
            .wrap(Logger::default())
            .wrap(actix_cors::Cors::permissive())
            .configure(routes::init_routes)
    })
    .bind(format!("{host}:{port}"))?
    .run()
    .await
}
