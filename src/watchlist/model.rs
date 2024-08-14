use crate::error_handler::CustomError;
use crate::schema::watchlists;
use crate::user;
use crate::{db::establish_connection, user::placeholder_user};
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
    pub fn find_by_media_type(media_type: String) -> Result<Self, CustomError> {
        let connection = &mut establish_connection();
        let watchlist = watchlists::table
            .select(Watchlist::as_select())
            .filter(
                watchlists::user_id
                    .eq(placeholder_user())
                    .and(watchlists::media_type.eq(media_type.clone())),
            )
            .first(connection);

        if let Ok(existing_watchlist) = watchlist {
            return Ok(existing_watchlist);
        }

        let name = match media_type.as_str() {
            "movie" => "Movie Watchlist",
            "show" => "Shows Watchlist",
            _ => "Watchlist",
        }
        .to_string();

        let new_watchlist = Watchlist::create(NewWatchlist { name, media_type })?;
        return Ok(new_watchlist);
    }

    pub fn find_by_user(user_id: Uuid) -> Result<Vec<Self>, CustomError> {
        let connection = &mut establish_connection();
        let watchlists = watchlists::table
            .filter(watchlists::user_id.eq(user_id))
            .order(watchlists::name.desc())
            .select(Watchlist::as_select())
            .load(connection)
            .expect("Error loading watchlists");
        Ok(watchlists)
    }

    pub fn create(watchlist: NewWatchlist) -> Result<Self, CustomError> {
        let watchlist_to_save = Watchlist {
            watchlist_id: Uuid::new_v4(),
            user_id: placeholder_user(),
            name: watchlist.name,
            media_type: watchlist.media_type,
        };

        let connection = &mut establish_connection();
        let new_watchlist = diesel::insert_into(watchlists::table)
            .values(watchlist_to_save)
            .get_result(connection)
            .expect("Error creating watchlist");
        Ok(new_watchlist)
    }

    pub fn update(id: Uuid, watchlist: NewWatchlist) -> Result<Self, CustomError> {
        let watchlist_to_save = Watchlist {
            watchlist_id: id,
            user_id: placeholder_user(),
            name: watchlist.name,
            media_type: watchlist.media_type,
        };

        let connection = &mut establish_connection();
        let updated_watchlist = diesel::update(watchlists::table)
            .filter(watchlists::watchlist_id.eq(id))
            .set(watchlist_to_save)
            .get_result(connection)
            .expect("Error updating watchlist");
        Ok(updated_watchlist)
    }

    pub fn delete(watchlist_id: Uuid) -> Result<usize, CustomError> {
        let connection = &mut establish_connection();
        let res =
            diesel::delete(watchlists::table.filter(watchlists::watchlist_id.eq(watchlist_id)))
                .execute(connection)
                .expect("Error deleting watchlist");
        Ok(res)
    }
}
