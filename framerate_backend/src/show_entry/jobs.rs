use std::{env, time::Duration};

use crate::{db::DbPool, show_entry, tmdb::TmdbClient, utils::env_vars};
use actix_web::rt::{spawn, time};
use tracing::{info, warn};

pub fn create_show_entry_metadata_updater(pool: DbPool, job_client: TmdbClient) {
    let job_interval = env::var("ENTRY_METADATA_JOB_INTERVAL")
        .ok()
        .and_then(|port| port.parse::<u64>().ok())
        .unwrap_or(0);

    let outdated_delta = env::var("SHOW_ENTRY_OUTDATED_DURATION")
        .ok()
        .and_then(|delta| env_vars::parse_time_delta_variable(&delta))
        .unwrap_or(chrono::Duration::weeks(6));

    if job_interval == 0 {
        warn!(target: "Entry Updater (Show)", "Skipping setup");
        return;
    }

    info!(target: "Entry Updater (Show)", "Creating entry updater job with interval of {job_interval:?} seconds and delta of {outdated_delta:?}");

    spawn(async move {
        let mut interval = time::interval(Duration::from_secs(job_interval));
        let mut previous_show_id = 0;

        loop {
            interval.tick().await;
            let mut conn = pool.get().unwrap();

            let entry = show_entry::ShowEntry::internal_find_outdated(&mut conn, outdated_delta);
            if let Ok(entry) = entry {
                if previous_show_id != entry.show_id {
                    previous_show_id = entry.show_id;
                } else {
                    continue;
                }
                match entry.internal_update_status(&mut conn, &job_client).await {
                    Ok(updated) => {
                        info!(target: "Entry Updater (Show)",
                            "Updated status for entry {} ({})",
                            updated.show_id, updated.name
                        );
                    }
                    Err(e) => {
                        // TODO: handle potential infinite loop if update fails
                        interval.reset_after(Duration::from_secs(86400));
                        warn!(target: "Entry Updater (Show)", "Error updating status: {}", e);
                    }
                }
            } else {
                info!(target: "Entry Updater (Show)", "No outdated entries found");
                interval.reset_after(Duration::from_secs(86400));
            }
        }
    });
}
