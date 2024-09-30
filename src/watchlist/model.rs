use crate::db::establish_connection;
use crate::error_handler::CustomError;
use crate::schema::watchlists;
use crate::user;
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

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NewWatchlist {
    pub name: String,
    pub media_type: String,
}

impl Watchlist {
    pub fn find_by_media_type(user_id: Uuid, media_type: String) -> Result<Self, CustomError> {
        let connection = &mut establish_connection();
        let watchlist = watchlists::table
            .select(Watchlist::as_select())
            .filter(
                watchlists::user_id
                    .eq(user_id)
                    .and(watchlists::media_type.eq(media_type.clone())),
            )
            .first(connection);

        if let Ok(existing_watchlist) = watchlist {
            return Ok(existing_watchlist);
        }

        let name = match media_type.as_str() {
            "movie" => "Movie Watchlist",
            "show" => "Show Watchlist",
            _ => "Watchlist",
        }
        .to_string();

        let new_watchlist = Self::create(Watchlist {
            watchlist_id: Uuid::new_v4(),
            media_type,
            user_id,
            name,
        })?;
        Ok(new_watchlist)
    }

    pub fn find_by_user(user_id: Uuid) -> Result<Vec<Self>, CustomError> {
        let connection = &mut establish_connection();
        let watchlists = watchlists::table
            .filter(watchlists::user_id.eq(user_id))
            .order(watchlists::name.desc())
            .select(Watchlist::as_select())
            .load(connection)?;
        Ok(watchlists)
    }

    pub fn create(watchlist: Watchlist) -> Result<Self, CustomError> {
        let connection = &mut establish_connection();
        let new_watchlist = diesel::insert_into(watchlists::table)
            .values(watchlist)
            .get_result(connection)?;
        Ok(new_watchlist)
    }

    pub fn update(watchlist: Watchlist) -> Result<Self, CustomError> {
        let connection = &mut establish_connection();
        let updated_watchlist = diesel::update(watchlists::table)
            .filter(watchlists::watchlist_id.eq(watchlist.watchlist_id))
            .set(watchlist)
            .get_result(connection)?;
        Ok(updated_watchlist)
    }

    pub fn delete(watchlist_id: Uuid) -> Result<usize, CustomError> {
        let connection = &mut establish_connection();
        let res =
            diesel::delete(watchlists::table.filter(watchlists::watchlist_id.eq(watchlist_id)))
                .execute(connection)?;
        Ok(res)
    }
}
