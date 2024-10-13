use crate::error_handler::CustomError;
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
    #[serde(skip_serializing_if = "Option::is_none")]
    pub imdb_id: Option<String>,
    pub user_id: Uuid,
    pub media_type: String,
    pub media_title: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub media_poster_uri: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
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
            .load(connection)?;
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
            .first(connection)?;
        Ok(watchlist_entries)
    }

    pub fn create(watchlist_entry: WatchlistEntry) -> Result<Self, CustomError> {
        let connection = &mut establish_connection();
        let new_watchlist = diesel::insert_into(watchlist_entries::table)
            .values(watchlist_entry)
            .get_result(connection)?;
        Ok(new_watchlist)
    }

    pub fn delete(watchlist_id: Uuid, media_id: i32) -> Result<usize, CustomError> {
        let connection = &mut establish_connection();
        let res = diesel::delete(
            watchlist_entries::table.filter(
                watchlist_entries::watchlist_id
                    .eq(watchlist_id)
                    .and(watchlist_entries::media_id.eq(media_id)),
            ),
        )
        .execute(connection)?;
        Ok(res)
    }
}
