use crate::{db::DbPool, show_entry, tmdb::TmdbClient};
use actix_web::rt::{spawn, time};
use std::{env, time::Duration};
use tracing::info;

pub fn create_show_entry_updater(pool: DbPool, job_client: TmdbClient) {
    let job_interval = env::var("JOB_INTERVAL")
        .ok()
        .and_then(|port| port.parse::<u64>().ok())
        .unwrap_or(0);

    if job_interval == 0 {
        return;
    }

    info!(target: "Entry Updater", "Creating entry updater job with interval of {job_interval:?} seconds");

    spawn(async move {
        let mut interval = time::interval(Duration::from_secs(job_interval));
        let mut previous_show_id = 0;

        loop {
            interval.tick().await;
            let mut conn = pool.get().unwrap();

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
                        interval.reset_after(Duration::from_secs(86400));
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
