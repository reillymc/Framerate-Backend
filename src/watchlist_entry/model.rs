use crate::error_handler::CustomError;
use crate::user::placeholder_user;
use crate::watchlist::Watchlist;
use crate::{db::establish_connection, schema::watchlist_entries};
use crate::{user, watchlist};
use chrono::NaiveDate;
use diesel::prelude::*;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Serialize, Deserialize, AsChangeset, Insertable, Associations, Selectable, Queryable)]
#[diesel(belongs_to(user::User))]
#[diesel(belongs_to(watchlist::Watchlist))]
#[diesel(table_name = watchlist_entries)]
#[serde(rename_all = "camelCase")]
pub struct WatchlistEntry {
    pub watchlist_id: Uuid,
    pub media_id: i32,
    pub imdb_id: Option<String>,
    pub user_id: Uuid,
    pub media_type: String,
    pub media_title: String,
    pub media_poster_uri: Option<String>,
    pub media_release_date: Option<NaiveDate>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NewWatchlistEntry {
    pub media_id: i32,
    pub imdb_id: Option<String>,
    pub media_type: String,
    pub media_title: String,
    pub media_poster_uri: Option<String>,
    pub media_release_date: Option<NaiveDate>,
}

impl WatchlistEntry {
    pub fn find_by_user_and_list(
        user_id: Uuid,
        media_type: String,
    ) -> Result<Vec<Self>, CustomError> {
        let connection = &mut establish_connection();
        let watchlist_entries = watchlist_entries::table
            .filter(
                watchlist_entries::user_id
                    .eq(user_id)
                    .and(watchlist_entries::media_type.eq(media_type)),
            )
            .order(watchlist_entries::media_release_date.desc())
            .select(WatchlistEntry::as_select())
            .load(connection)
            .expect("Error loading watchlists");
        Ok(watchlist_entries)
    }

    pub fn find_entry(
        user_id: Uuid,
        media_type: String,
        media_id: i32,
    ) -> Result<Self, CustomError> {
        let connection = &mut establish_connection();
        let watchlist_entries = watchlist_entries::table
            .filter(
                watchlist_entries::user_id
                    .eq(user_id)
                    .and(watchlist_entries::media_type.eq(media_type))
                    .and(watchlist_entries::media_id.eq(media_id)),
            )
            .order(watchlist_entries::media_release_date.desc())
            .select(WatchlistEntry::as_select())
            .first(connection)
            .expect("Error loading watchlists");
        Ok(watchlist_entries)
    }

    pub fn create(
        media_type: String,
        watchlist_entry: NewWatchlistEntry,
    ) -> Result<Self, CustomError> {
        let watchlist = Watchlist::find_by_media_type(media_type.clone())?;

        let watchlist_entry_to_save = WatchlistEntry {
            watchlist_id: watchlist.watchlist_id,
            user_id: placeholder_user(),
            media_id: watchlist_entry.media_id,
            imdb_id: watchlist_entry.imdb_id,
            media_type: watchlist_entry.media_type,
            media_title: watchlist_entry.media_title,
            media_poster_uri: watchlist_entry.media_poster_uri,
            media_release_date: watchlist_entry.media_release_date,
        };

        let connection = &mut establish_connection();
        let new_watchlist = diesel::insert_into(watchlist_entries::table)
            .values(watchlist_entry_to_save)
            .get_result(connection)
            .expect("Error creating watchlist");
        Ok(new_watchlist)
    }

    pub fn update(
        media_type: String,
        watchlist_entry: NewWatchlistEntry,
    ) -> Result<Self, CustomError> {
        let watchlist = Watchlist::find_by_media_type(media_type.clone())?;

        let watchlist_entry_to_save = WatchlistEntry {
            watchlist_id: watchlist.watchlist_id,
            user_id: placeholder_user(),
            media_id: watchlist_entry.media_id,
            imdb_id: watchlist_entry.imdb_id,
            media_type: watchlist_entry.media_type,
            media_title: watchlist_entry.media_title,
            media_poster_uri: watchlist_entry.media_poster_uri,
            media_release_date: watchlist_entry.media_release_date,
        };

        let connection = &mut establish_connection();
        let updated_watchlist = diesel::update(watchlist_entries::table)
            .filter(
                watchlist_entries::media_type
                    .eq(media_type)
                    .and(watchlist_entries::media_id.eq(watchlist_entry.media_id)),
            )
            .set(watchlist_entry_to_save)
            .get_result(connection)
            .expect("Error updating watchlist");
        Ok(updated_watchlist)
    }

    pub fn delete(media_type: String, media_id: i32) -> Result<usize, CustomError> {
        let watchlist = Watchlist::find_by_media_type(media_type.clone())?;

        let connection = &mut establish_connection();
        let res = diesel::delete(
            watchlist_entries::table.filter(
                watchlist_entries::media_type
                    .eq(media_type)
                    .and(watchlist_entries::watchlist_id.eq(watchlist.watchlist_id))
                    .and(watchlist_entries::media_id.eq(media_id)),
            ),
        )
        .execute(connection)
        .expect("Error deleting watchlist");
        Ok(res)
    }
}
