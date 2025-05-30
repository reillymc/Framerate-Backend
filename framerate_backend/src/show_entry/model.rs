use crate::db::DbConnection;
use crate::schema::show_entries;
use crate::show::{Show, SHOW_ACTIVE_STATUSES};
use crate::tmdb::TmdbClient;
use crate::utils::AppError;
use crate::{collection, user};
use chrono::{Duration, NaiveDate, TimeDelta, Utc};
use diesel::prelude::*;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::Uuid;

#[derive(
    Serialize,
    Deserialize,
    AsChangeset,
    Insertable,
    Associations,
    Selectable,
    Queryable,
    Debug,
    PartialEq,
    ToSchema,
)]
#[diesel(belongs_to(user::User))]
#[diesel(belongs_to(collection::Collection))]
#[diesel(table_name = show_entries)]
#[serde(rename_all = "camelCase")]
pub struct ShowEntry {
    pub collection_id: Uuid,
    pub show_id: i32,
    pub user_id: Uuid,
    pub name: String,
    pub updated_at: NaiveDate,
    #[schema(nullable = false)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub imdb_id: Option<String>,
    #[schema(nullable = false)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub status: Option<String>,
    #[schema(nullable = false)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub poster_path: Option<String>,
    #[schema(nullable = false)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub first_air_date: Option<NaiveDate>,
    #[schema(nullable = false)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub last_air_date: Option<NaiveDate>,
    #[schema(nullable = false)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub next_air_date: Option<NaiveDate>,
}

impl ShowEntry {
    pub fn find_all(
        conn: &mut DbConnection,
        user_id: Uuid,
        collection_id: Uuid,
    ) -> Result<Vec<Self>, AppError> {
        let show_entries = show_entries::table
            .filter(show_entries::user_id.eq(user_id))
            .filter(show_entries::collection_id.eq(collection_id))
            .order(show_entries::first_air_date.desc())
            .select(ShowEntry::as_select())
            .load(conn)?;
        Ok(show_entries)
    }

    pub fn find(
        conn: &mut DbConnection,
        user_id: Uuid,
        collection_id: Uuid,
        show_id: i32,
    ) -> Result<Self, AppError> {
        let show_entries = show_entries::table
            .filter(show_entries::user_id.eq(user_id))
            .filter(show_entries::collection_id.eq(collection_id))
            .filter(show_entries::show_id.eq(show_id))
            .order(show_entries::first_air_date.desc())
            .select(ShowEntry::as_select())
            .first(conn)?;
        Ok(show_entries)
    }

    pub fn create(conn: &mut DbConnection, watchlist_entry: ShowEntry) -> Result<Self, AppError> {
        let new_watchlist = diesel::insert_into(show_entries::table)
            .values(watchlist_entry)
            .get_result(conn)?;
        Ok(new_watchlist)
    }

    pub fn delete(
        conn: &mut DbConnection,
        collection_id: Uuid,
        show_id: i32,
    ) -> Result<usize, AppError> {
        let res = diesel::delete(
            show_entries::table.filter(
                show_entries::collection_id
                    .eq(collection_id)
                    .and(show_entries::show_id.eq(show_id)),
            ),
        )
        .execute(conn)?;
        Ok(res)
    }

    pub fn find_collections(
        conn: &mut DbConnection,
        user_id: Uuid,
        show_id: &i32,
    ) -> Result<Vec<Uuid>, AppError> {
        let entries = show_entries::table
            .filter(show_entries::user_id.eq(user_id))
            .filter(show_entries::show_id.eq(show_id))
            .select(show_entries::collection_id)
            .load(conn)?;

        Ok(entries)
    }

    pub fn internal_find_outdated(
        conn: &mut DbConnection,
        outdated_delta: TimeDelta,
    ) -> Result<Self, AppError> {
        let show_entries = show_entries::table
            .filter(
                show_entries::status
                    .eq_any(SHOW_ACTIVE_STATUSES)
                    .or(show_entries::status.is_null()),
            )
            .filter(
                show_entries::updated_at
                    .lt(Utc::now().date_naive() - outdated_delta)
                    .or(show_entries::next_air_date
                        .lt(Utc::now().date_naive() + Duration::days(1))
                        .and(
                            show_entries::updated_at
                                .lt(Utc::now().date_naive() - Duration::days(1)),
                        )),
            )
            .select(ShowEntry::as_select())
            .first(conn)?;

        Ok(show_entries)
    }

    pub async fn internal_update_status(
        mut self,
        conn: &mut DbConnection,
        client: &TmdbClient,
    ) -> Result<Self, AppError> {
        if let Ok(show) = Show::find(client, &self.show_id).await {
            self.last_air_date = show.last_air_date;
            self.next_air_date = show.next_air_date;
            self.poster_path = show.poster_path;
            self.status = show.status;
        };

        self.updated_at = Utc::now().naive_utc().date();

        let updated = diesel::update(show_entries::table)
            .filter(show_entries::show_id.eq(self.show_id))
            .set(self)
            .get_result(conn)?;

        Ok(updated)
    }
}
