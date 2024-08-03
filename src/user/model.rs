use crate::{db::establish_connection, error_handler::CustomError, schema::users};
use chrono::NaiveDateTime;
use diesel::prelude::*;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Serialize, Deserialize, Queryable, Selectable, AsChangeset, Insertable)]
#[diesel(table_name = users)]
pub struct User {
    pub user_id: Uuid,
    pub email: String,
    pub password: String,
    pub first_name: String,
    pub last_name: String,
    pub avatar_uri: Option<String>,
    pub date_created: NaiveDateTime,
    pub permission_level: i16,
    pub public: bool,
}

pub fn placeholder_user() -> Uuid {
    Uuid::parse_str("82986e28-47e7-4fb4-9c48-986f6e8715b4").unwrap_or_else(|_| Uuid::nil())
}
impl User {
    pub fn find(user_id: Uuid) -> Result<Self, CustomError> {
        let connection = &mut establish_connection();
        let users = users::table
            .select(User::as_select())
            .filter(users::user_id.eq(user_id))
            .first(connection)
            .expect("Error loading posts");
        Ok(users)
    }

    pub fn create(user: User) -> Result<Self, CustomError> {
        let connection = &mut establish_connection();
        let new_user = diesel::insert_into(users::table)
            .values(user)
            .get_result(connection)
            .expect("Error creating user");
        Ok(new_user)
    }

    pub fn update(id: Uuid, user: User) -> Result<Self, CustomError> {
        let connection = &mut establish_connection();
        let updated_user = diesel::update(users::table)
            .filter(users::user_id.eq(id))
            .set(user)
            .get_result(connection)
            .expect("Error updating user");
        Ok(updated_user)
    }

    pub fn delete(user_id: Uuid) -> Result<usize, CustomError> {
        let connection = &mut establish_connection();
        let res = diesel::delete(users::table.filter(users::user_id.eq(user_id)))
            .execute(connection)
            .expect("Error deleting user");
        Ok(res)
    }
}
