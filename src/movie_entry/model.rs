use crate::error_handler::CustomError;
use crate::{db::establish_connection, schema::movie_entries};
use crate::{user, watchlist};
use chrono::NaiveDate;
use diesel::prelude::*;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Serialize, Deserialize, AsChangeset, Insertable, Associations, Selectable, Queryable)]
#[diesel(belongs_to(user::User))]
#[diesel(belongs_to(watchlist::Watchlist))]
#[diesel(table_name = movie_entries)]
#[serde(rename_all = "camelCase")]
pub struct MovieEntry {
    pub watchlist_id: Uuid,
    pub movie_id: i32,
    pub user_id: Uuid,
    pub title: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub imdb_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub poster_path: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub release_date: Option<NaiveDate>,
}

impl MovieEntry {
    pub fn find_all(user_id: Uuid, watchlist_id: Uuid) -> Result<Vec<Self>, CustomError> {
        let connection = &mut establish_connection();
        let movie_entries = movie_entries::table
            .filter(movie_entries::user_id.eq(user_id))
            .filter(movie_entries::watchlist_id.eq(watchlist_id))
            .order(movie_entries::release_date.desc())
            .select(MovieEntry::as_select())
            .load(connection)?;
        Ok(movie_entries)
    }

    pub fn find(user_id: Uuid, watchlist_id: Uuid, movie_id: i32) -> Result<Self, CustomError> {
        let connection = &mut establish_connection();
        let movie_entries = movie_entries::table
            .filter(movie_entries::user_id.eq(user_id))
            .filter(movie_entries::watchlist_id.eq(watchlist_id))
            .filter(movie_entries::movie_id.eq(movie_id))
            .order(movie_entries::release_date.desc())
            .select(MovieEntry::as_select())
            .first(connection)?;
        Ok(movie_entries)
    }

    pub fn create(watchlist_entry: MovieEntry) -> Result<Self, CustomError> {
        let connection = &mut establish_connection();
        let new_watchlist = diesel::insert_into(movie_entries::table)
            .values(watchlist_entry)
            .get_result(connection)?;
        Ok(new_watchlist)
    }

    pub fn delete(watchlist_id: Uuid, movie_id: i32) -> Result<usize, CustomError> {
        let connection = &mut establish_connection();
        let res = diesel::delete(
            movie_entries::table.filter(
                movie_entries::watchlist_id
                    .eq(watchlist_id)
                    .and(movie_entries::movie_id.eq(movie_id)),
            ),
        )
        .execute(connection)?;
        Ok(res)
    }
}
