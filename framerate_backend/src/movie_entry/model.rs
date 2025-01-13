use crate::db::DbConnection;
use crate::movie::{Movie, MOVIE_ACTIVE_STATUSES};
use crate::schema::movie_entries;
use crate::tmdb::TmdbClient;
use crate::utils::AppError;
use crate::{collection, user};
use chrono::{NaiveDate, TimeDelta, Utc};
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
#[diesel(table_name = movie_entries)]
#[serde(rename_all = "camelCase")]
pub struct MovieEntry {
    pub collection_id: Uuid,
    pub movie_id: i32,
    pub user_id: Uuid,
    pub title: String,
    #[schema(nullable = false)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub imdb_id: Option<String>,
    #[schema(nullable = false)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub poster_path: Option<String>,
    #[schema(nullable = false)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub release_date: Option<NaiveDate>,
    #[schema(nullable = false)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub status: Option<String>,
    pub updated_at: NaiveDate,
}

impl MovieEntry {
    pub fn find_all(
        conn: &mut DbConnection,
        user_id: Uuid,
        collection_id: Uuid,
    ) -> Result<Vec<Self>, AppError> {
        let movie_entries = movie_entries::table
            .filter(movie_entries::user_id.eq(user_id))
            .filter(movie_entries::collection_id.eq(collection_id))
            .order(movie_entries::release_date.desc())
            .select(MovieEntry::as_select())
            .load(conn)?;
        Ok(movie_entries)
    }

    pub fn find(
        conn: &mut DbConnection,
        user_id: Uuid,
        collection_id: Uuid,
        movie_id: i32,
    ) -> Result<Self, AppError> {
        let movie_entries = movie_entries::table
            .filter(movie_entries::user_id.eq(user_id))
            .filter(movie_entries::collection_id.eq(collection_id))
            .filter(movie_entries::movie_id.eq(movie_id))
            .order(movie_entries::release_date.desc())
            .select(MovieEntry::as_select())
            .first(conn)?;
        Ok(movie_entries)
    }

    pub fn create(conn: &mut DbConnection, watchlist_entry: MovieEntry) -> Result<Self, AppError> {
        let new_watchlist = diesel::insert_into(movie_entries::table)
            .values(watchlist_entry)
            .get_result(conn)?;
        Ok(new_watchlist)
    }

    pub fn delete(
        conn: &mut DbConnection,
        collection_id: Uuid,
        movie_id: i32,
    ) -> Result<usize, AppError> {
        let res = diesel::delete(
            movie_entries::table.filter(
                movie_entries::collection_id
                    .eq(collection_id)
                    .and(movie_entries::movie_id.eq(movie_id)),
            ),
        )
        .execute(conn)?;
        Ok(res)
    }

    pub fn find_collections(
        conn: &mut DbConnection,
        user_id: Uuid,
        movie_id: &i32,
    ) -> Result<Vec<Uuid>, AppError> {
        let entries = movie_entries::table
            .filter(movie_entries::user_id.eq(user_id))
            .filter(movie_entries::movie_id.eq(movie_id))
            .select(movie_entries::collection_id)
            .load(conn)?;

        Ok(entries)
    }

    pub fn internal_find_outdated(
        conn: &mut DbConnection,
        outdated_delta: TimeDelta,
    ) -> Result<Self, AppError> {
        let movie_entries = movie_entries::table
            .filter(
                movie_entries::status
                    .eq_any(MOVIE_ACTIVE_STATUSES)
                    .or(movie_entries::status.is_null()),
            )
            .filter(movie_entries::updated_at.lt(Utc::now().date_naive() - outdated_delta))
            .select(MovieEntry::as_select())
            .first(conn)?;

        Ok(movie_entries)
    }

    pub async fn internal_update_status(
        mut self,
        conn: &mut DbConnection,
        client: &TmdbClient,
    ) -> Result<Self, AppError> {
        if let Ok(movie) = Movie::find(client, &self.movie_id).await {
            self.release_date = movie.release_date;
            self.poster_path = movie.poster_path;
            self.status = movie.status;
        };

        self.updated_at = Utc::now().naive_utc().date();

        let updated = diesel::update(movie_entries::table)
            .filter(movie_entries::movie_id.eq(self.movie_id))
            .set(self)
            .get_result(conn)?;

        Ok(updated)
    }
}
