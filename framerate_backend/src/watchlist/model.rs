use crate::db::DbConnection;
use crate::schema::watchlists;
use crate::user;
use crate::utils::AppError;
use diesel::prelude::*;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Serialize, Deserialize, AsChangeset, Insertable, Associations, Selectable, Queryable)]
#[diesel(belongs_to(user::User))]
#[diesel(table_name = watchlists)]
#[serde(rename_all = "camelCase")]
pub struct Watchlist {
    pub watchlist_id: Uuid,
    pub user_id: Uuid,
    pub name: String,
    pub media_type: String,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct NewWatchlist {
    pub name: String,
    pub media_type: String,
}

impl Watchlist {
    pub fn find_default(
        conn: &mut DbConnection,
        user_id: Uuid,
        media_type: &str,
    ) -> Result<Self, AppError> {
        let watchlist = watchlists::table
            .select(Watchlist::as_select())
            .filter(
                watchlists::user_id
                    .eq(user_id)
                    .and(watchlists::media_type.eq(media_type)),
            )
            .first(conn);

        if let Ok(existing_watchlist) = watchlist {
            return Ok(existing_watchlist);
        }

        let name = match media_type {
            "movie" => "Movie Watchlist",
            "show" => "Show Watchlist",
            _ => "Watchlist",
        }
        .to_string();

        let new_watchlist = Self::create(
            conn,
            Watchlist {
                watchlist_id: Uuid::new_v4(),
                media_type: media_type.to_string(),
                user_id,
                name,
            },
        )?;
        Ok(new_watchlist)
    }

    pub fn find_by_user(conn: &mut DbConnection, user_id: Uuid) -> Result<Vec<Self>, AppError> {
        let watchlists = watchlists::table
            .filter(watchlists::user_id.eq(user_id))
            .order(watchlists::name.desc())
            .select(Watchlist::as_select())
            .load(conn)?;
        Ok(watchlists)
    }

    pub fn create(conn: &mut DbConnection, watchlist: Watchlist) -> Result<Self, AppError> {
        let new_watchlist = diesel::insert_into(watchlists::table)
            .values(watchlist)
            .get_result(conn)?;
        Ok(new_watchlist)
    }

    pub fn update(conn: &mut DbConnection, watchlist: Watchlist) -> Result<Self, AppError> {
        let updated_watchlist = diesel::update(watchlists::table)
            .filter(watchlists::watchlist_id.eq(watchlist.watchlist_id))
            .set(watchlist)
            .get_result(conn)?;
        Ok(updated_watchlist)
    }

    pub fn delete(conn: &mut DbConnection, watchlist_id: Uuid) -> Result<usize, AppError> {
        let res =
            diesel::delete(watchlists::table.filter(watchlists::watchlist_id.eq(watchlist_id)))
                .execute(conn)?;
        Ok(res)
    }
}
