extern crate diesel;
// #[macro_use]
extern crate diesel_migrations;

use actix_web::{middleware::Logger, App, HttpServer};
use db::establish_connection;
// use db::establish_connection;
use dotenvy::dotenv;
use env_logger::Env;
use listenfd::ListenFd;
use std::env;

mod db;
mod error_handler;
mod movie;
mod review;
mod review_company;
mod routes;
mod schema;
mod user;
mod watchlist;
mod watchlist_entry;

#[actix_rt::main]
async fn main() -> std::io::Result<()> {
    dotenv().ok();

    let connection = &mut establish_connection();
    db::run_db_migrations(connection);

    env_logger::init_from_env(Env::default().default_filter_or("info"));

    let mut listenfd = ListenFd::from_env();
    let mut server = HttpServer::new(|| {
        App::new()
            .wrap(Logger::default())
            .wrap(Logger::new("%a %{User-Agent}i"))
            .wrap(actix_cors::Cors::permissive())
            .configure(routes::init_routes)
    });

    server = if let Some(listener) = listenfd.take_tcp_listener(0)? {
        server.listen(listener)?
    } else {
        let host = env::var("HOST").expect("Please set host in .env");
        let port = env::var("PORT").expect("Please set port in .env");
        server.bind(format!("{host}:{port}"))?
    };

    // log ip to console
    println!(
        "Server running at http://{}:{}",
        env::var("HOST").unwrap(),
        env::var("PORT").unwrap()
    );

    server.run().await
}
