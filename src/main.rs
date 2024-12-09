extern crate diesel;
// #[macro_use]
extern crate diesel_migrations;

use actix_cors::Cors;
use actix_web::{
    middleware::Logger,
    rt::{spawn, time},
    web::Data,
    App, HttpServer,
};
use std::{env, time::Duration};
use tracing::info;

use framerate::{db, log, routes, show_entry, tmdb};

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    log::setup_logger();
    let pool = db::get_connection_pool();
    let mut conn = pool.get().unwrap();
    db::run_db_migrations(&mut conn);

    // Don't use caching until appropriate clean-up solution is implemented
    let client = tmdb::get_client(false);

    let host = env::var("HOST").unwrap_or("localhost".to_string());
    let port = env::var("PORT").unwrap_or("3000".to_string());

    let job_interval = env::var("JOB_INTERVAL")
        .ok()
        .and_then(|port| port.parse::<u64>().ok())
        .unwrap_or(3600);

    if job_interval != 0 {
        let job_client = client.clone();
        spawn(async move {
            info!(target: "Entry Updater", "Creating entry updater job with interval of {job_interval:?} seconds");

            let mut interval = time::interval(Duration::from_secs(job_interval));
            let mut previous_show_id = 0;
            loop {
                interval.tick().await;
                let entry = show_entry::ShowEntry::internal_find_outdated(&mut conn);
                if let Ok(entry) = entry {
                    if previous_show_id != entry.show_id {
                        previous_show_id = entry.show_id;
                    } else {
                        continue;
                    }
                    match entry.internal_update_status(&mut conn, &job_client).await {
                        Ok(updated) => {
                            info!(target: "Entry Updater",
                                "Updated status for entry {} ({})",
                                updated.show_id, updated.name
                            );
                        }
                        Err(e) => {
                            // TODO: handle potential infinite loop if update fails
                            info!(target: "Entry Updater", "Error updating status: {}", e);
                        }
                    }
                } else {
                    info!(target: "Entry Updater", "No outdated entries found");
                    interval.reset_after(Duration::from_secs(86400));
                }
            }
        });
    }

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
