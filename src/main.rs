extern crate diesel;
// #[macro_use]
extern crate diesel_migrations;

use actix_cors::Cors;
use actix_web::{
    middleware::Logger,
    rt::{spawn, time},
    App, HttpServer,
};
use db::establish_connection;
use env_logger::Env;
use show_entry::ShowEntry;
use std::{env, time::Duration};

mod authentication;
mod db;
mod error_handler;
mod movie;
mod movie_entry;
mod movie_review;
mod review;
mod review_company;
mod routes;
mod schema;
mod season;
mod season_review;
mod show;
mod show_entry;
mod show_review;
mod user;
mod utils;
mod watchlist;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let connection = &mut establish_connection();
    db::run_db_migrations(connection);

    env_logger::init_from_env(Env::default().default_filter_or("info"));

    let host = env::var("HOST").unwrap_or("localhost".to_string());
    let port = env::var("PORT").unwrap_or("3000".to_string());
    println!("Server starting at http://{host}:{port}");

    spawn(async move {
        let mut interval = time::interval(Duration::from_secs(10));
        let mut previous_show_id = 0;
        loop {
            interval.tick().await;
            let entry = ShowEntry::internal_find_outdated();
            if let Ok(entry) = entry {
                if previous_show_id != entry.show_id {
                    previous_show_id = entry.show_id;
                } else {
                    println!(
                        "Skipping update of show entry {} ({})",
                        entry.show_id, entry.name
                    );
                    continue;
                }
                match entry.internal_update_status().await {
                    Ok(updated) => {
                        println!(
                            "Updated status for entry {} ({})",
                            updated.show_id, updated.name
                        );
                    }
                    Err(e) => {
                        // TODO: handle potential infinite loop if update fails
                        println!("Error updating status: {}", e);
                    }
                }
            } else {
                println!("No outdated entries found");
                interval.reset_after(Duration::from_secs(86400));
            }
        }
    });

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
