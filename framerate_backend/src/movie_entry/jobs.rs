use crate::{db::DbPool, movie_entry, tmdb::TmdbClient, utils::env_vars};
use actix_web::rt::{spawn, time};
use std::{env, time::Duration};
use tracing::{info, warn};

pub fn create_movie_entry_metadata_updater(pool: DbPool, job_client: TmdbClient) {
    let job_interval = env::var("ENTRY_METADATA_JOB_INTERVAL")
        .ok()
        .and_then(|port| port.parse::<u64>().ok())
        .unwrap_or(0);

    let outdated_delta = env::var("MOVIE_ENTRY_OUTDATED_DURATION")
        .ok()
        .and_then(|delta| env_vars::parse_time_delta_variable(&delta))
        .unwrap_or(chrono::Duration::weeks(8));

    if job_interval == 0 {
        warn!(target: "Entry Updater (Movie)", "Skipping setup");
        return;
    }

    info!(target: "Entry Updater (Movie)", "Creating entry updater job with interval of {job_interval:?} seconds and delta of {outdated_delta:?}");

    spawn(async move {
        let mut interval = time::interval(Duration::from_secs(job_interval));
        let mut previous_movie_id = 0;

        loop {
            interval.tick().await;
            let mut conn = pool.get().unwrap();

            let entry = movie_entry::MovieEntry::internal_find_outdated(&mut conn, outdated_delta);
            if let Ok(entry) = entry {
                if previous_movie_id != entry.movie_id {
                    previous_movie_id = entry.movie_id;
                } else {
                    continue;
                }
                match entry.internal_update_status(&mut conn, &job_client).await {
                    Ok(updated) => {
                        info!(target: "Entry Updater (Movie)",
                            "Updated status for entry {} ({})",
                            updated.movie_id, updated.title
                        );
                    }
                    Err(e) => {
                        // TODO: handle potential infinite loop if update fails
                        interval.reset_after(Duration::from_secs(86400));
                        warn!(target: "Entry Updater (Movie)", "Error updating status: {}", e);
                    }
                }
            } else {
                info!(target: "Entry Updater (Movie)", "No outdated entries found");
                interval.reset_after(Duration::from_secs(86400));
            }
        }
    });
}
