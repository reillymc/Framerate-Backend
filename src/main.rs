extern crate diesel;
// #[macro_use]
extern crate diesel_migrations;

use actix_cors::Cors;
use actix_web::{middleware::Logger, web::Data, App, HttpServer};
use std::env;
use tracing::info;
use tracing_log::LogTracer;

use framerate::{db, jobs, routes, tmdb, utils};

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    LogTracer::init().ok();
    utils::log::setup_logger();

    let pool = db::get_connection_pool();
    let mut conn = pool.get().unwrap();
    db::run_db_migrations(&mut conn);

    // Don't use caching until appropriate clean-up solution is implemented
    let client = tmdb::get_client(false);

    jobs::create_show_entry_updater(pool.clone(), client.clone());

    let host = env::var("HOST").unwrap_or("localhost".to_string());
    let port = env::var("PORT").unwrap_or("3000".to_string());

    info!("Server starting at http://{host}:{port}");

    HttpServer::new(move || {
        App::new()
            .app_data(Data::new(pool.clone()))
            .app_data(Data::new(client.clone()))
            .wrap(Cors::default())
            .wrap(Logger::default())
            .wrap(actix_cors::Cors::permissive())
            .configure(routes::init_routes)
    })
    .bind(format!("{host}:{port}"))?
    .run()
    .await
}
