use crate::error_handler::CustomError;
use crate::show::{Show, SHOW_ACTIVE_STATUSES};
use crate::{db::establish_connection, schema::show_entries};
use crate::{user, watchlist};
use chrono::{Duration, NaiveDate, Utc};
use diesel::prelude::*;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(
    Serialize, Deserialize, AsChangeset, Insertable, Associations, Selectable, Queryable, Debug,
)]
#[diesel(belongs_to(user::User))]
#[diesel(belongs_to(watchlist::Watchlist))]
#[diesel(table_name = show_entries)]
#[serde(rename_all = "camelCase")]
pub struct ShowEntry {
    pub watchlist_id: Uuid,
    pub show_id: i32,
    pub user_id: Uuid,
    pub name: String,
    pub updated_at: NaiveDate,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub imdb_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub status: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub poster_path: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub first_air_date: Option<NaiveDate>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub last_air_date: Option<NaiveDate>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub next_air_date: Option<NaiveDate>,
}

impl ShowEntry {
    pub fn find_all(user_id: Uuid, watchlist_id: Uuid) -> Result<Vec<Self>, CustomError> {
        let connection = &mut establish_connection();
        let show_entries = show_entries::table
            .filter(show_entries::user_id.eq(user_id))
            .filter(show_entries::watchlist_id.eq(watchlist_id))
            .order(show_entries::first_air_date.desc())
            .select(ShowEntry::as_select())
            .load(connection)?;
        Ok(show_entries)
    }

    pub fn find(user_id: Uuid, watchlist_id: Uuid, show_id: i32) -> Result<Self, CustomError> {
        let connection = &mut establish_connection();
        let show_entries = show_entries::table
            .filter(show_entries::user_id.eq(user_id))
            .filter(show_entries::watchlist_id.eq(watchlist_id))
            .filter(show_entries::show_id.eq(show_id))
            .order(show_entries::first_air_date.desc())
            .select(ShowEntry::as_select())
            .first(connection)?;
        Ok(show_entries)
    }

    pub fn create(watchlist_entry: ShowEntry) -> Result<Self, CustomError> {
        let connection = &mut establish_connection();
        let new_watchlist = diesel::insert_into(show_entries::table)
            .values(watchlist_entry)
            .get_result(connection)?;
        Ok(new_watchlist)
    }

    pub fn delete(watchlist_id: Uuid, show_id: i32) -> Result<usize, CustomError> {
        let connection = &mut establish_connection();
        let res = diesel::delete(
            show_entries::table.filter(
                show_entries::watchlist_id
                    .eq(watchlist_id)
                    .and(show_entries::show_id.eq(show_id)),
            ),
        )
        .execute(connection)?;
        Ok(res)
    }

    pub fn internal_find_outdated() -> Result<Self, CustomError> {
        let connection = &mut establish_connection();

        let show_entries =
            show_entries::table
                .filter(show_entries::status.eq_any(SHOW_ACTIVE_STATUSES))
                .filter(
                    show_entries::updated_at
                        .lt(Utc::now().date_naive() - Duration::weeks(6))
                        .or(show_entries::next_air_date
                            .lt(Utc::now().date_naive() + Duration::days(1))),
                )
                .select(ShowEntry::as_select())
                .first(connection)?;

        Ok(show_entries)
    }

    pub async fn internal_update_status(mut self) -> Result<Self, CustomError> {
        if let Ok(show) = Show::find(&self.show_id).await {
            self.last_air_date = show.last_air_date;
            self.next_air_date = show.next_air_date;
            self.poster_path = show.poster_path;
            self.status = show.status;
        };

        self.updated_at = Utc::now().naive_utc().date();

        let connection = &mut establish_connection();
        let updated = diesel::update(show_entries::table)
            .filter(show_entries::show_id.eq(self.show_id))
            .set(self)
            .get_result(connection)?;

        return Ok(updated);
    }
}
