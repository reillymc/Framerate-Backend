use crate::{db::establish_connection, error_handler::CustomError, schema::users};
use chrono::NaiveDateTime;
use diesel::prelude::*;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Serialize, Deserialize, Queryable, Selectable, AsChangeset, Insertable)]
#[diesel(table_name = users)]
#[serde(rename_all = "camelCase")]
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
    pub configuration: serde_json::Value,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NewUser {
    pub user_id: Option<Uuid>,
    pub email: String,
    pub password: String,
    pub first_name: String,
    pub last_name: String,
    pub avatar_uri: Option<String>,
    pub configuration: serde_json::Value,
}

#[derive(Debug, Clone, Deserialize, AsChangeset)]
#[diesel(table_name = users)]
#[serde(rename_all = "camelCase")]
pub struct UpdatedUser {
    pub user_id: Uuid,
    pub configuration: serde_json::Value,
}

#[derive(Serialize, Debug, Clone, Queryable, Deserialize, AsChangeset)]
#[diesel(table_name = users)]
#[serde(rename_all = "camelCase")]
pub struct UserResponse {
    pub user_id: Uuid,
    pub first_name: String,
    pub last_name: String,
    pub avatar_uri: Option<String>,
}

#[derive(Serialize, Debug, Clone, Queryable, Deserialize, AsChangeset)]
#[diesel(table_name = users)]
#[serde(rename_all = "camelCase")]
pub struct UserFindResponse {
    pub user_id: Uuid,
    pub email: String,
    pub first_name: String,
    pub last_name: String,
    pub avatar_uri: Option<String>,
    pub configuration: serde_json::Value,
}

pub fn placeholder_user() -> Uuid {
    Uuid::parse_str("82986e28-47e7-4fb4-9c48-986f6e8715b4").unwrap_or_else(|_| Uuid::nil())
}

impl User {
    pub fn find(user_id: Uuid) -> Result<UserFindResponse, CustomError> {
        let connection = &mut establish_connection();
        let users = users::table
            .select((
                users::user_id,
                users::email,
                users::first_name,
                users::last_name,
                users::avatar_uri,
                users::configuration,
            ))
            .filter(users::user_id.eq(user_id))
            .first(connection)
            .expect("Error loading posts");
        Ok(users)
    }

    pub fn find_all() -> Result<Vec<UserResponse>, CustomError> {
        let connection = &mut establish_connection();
        let users = users::table
            .select((
                users::user_id,
                users::first_name,
                users::last_name,
                users::avatar_uri,
            ))
            .load(connection)
            .expect("Error loading posts");
        Ok(users)
    }

    pub fn create(user: NewUser) -> Result<Self, CustomError> {
        let user_to_save = User {
            user_id: user.user_id.unwrap_or(Uuid::new_v4()),
            email: user.email,
            password: user.password,
            first_name: user.first_name,
            last_name: user.last_name,
            date_created: chrono::Local::now().naive_local(),
            permission_level: 0,
            public: false,
            avatar_uri: None,
            configuration: serde_json::json!({
                "people": [],
                "venues": [],
            }),
        };

        let connection = &mut establish_connection();
        let new_user = diesel::insert_into(users::table)
            .values(user_to_save)
            .get_result(connection)
            .expect("Error creating user");
        Ok(new_user)
    }

    pub fn update(id: Uuid, user: UpdatedUser) -> Result<Self, CustomError> {
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
