use crate::{
    db::establish_connection,
    error_handler::{AuthError, CustomError},
    schema::users,
};
use bcrypt::{hash, verify, DEFAULT_COST};
use chrono::NaiveDateTime;
use diesel::{dsl, prelude::*};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Serialize, Deserialize, Queryable, Selectable, AsChangeset, Insertable)]
#[diesel(table_name = users)]
#[serde(rename_all = "camelCase")]
pub struct User {
    pub user_id: Uuid,
    pub email: Option<String>,
    #[serde(skip)]
    pub password: Option<String>,
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
    pub email: Option<String>,
    pub password: Option<String>,
    pub first_name: String,
    pub last_name: String,
    pub avatar_uri: Option<String>,
    pub configuration: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Deserialize, AsChangeset)]
#[diesel(table_name = users)]
#[serde(rename_all = "camelCase")]
pub struct UpdatedUser {
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
    pub email: Option<String>,
    pub first_name: String,
    pub last_name: String,
    pub avatar_uri: Option<String>,
    pub configuration: serde_json::Value,
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
            .first(connection)?;
        Ok(users)
    }

    pub fn find_summary(user_id: Uuid) -> Result<UserResponse, CustomError> {
        let connection = &mut establish_connection();
        let users = users::table
            .select((
                users::user_id,
                users::first_name,
                users::last_name,
                users::avatar_uri,
            ))
            .filter(users::user_id.eq(user_id))
            .first(connection)?;
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
            .load(connection)?;
        Ok(users)
    }

    pub fn create(user: NewUser) -> Result<Self, CustomError> {
        let password = if let Some(pwd) = user.password {
            Some(Self::hash_password(pwd)?)
        } else {
            None
        };

        let user_to_save = User {
            user_id: user.user_id.unwrap_or(Uuid::new_v4()),
            email: user.email,
            password,
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
            .get_result(connection)?;
        Ok(new_user)
    }

    pub fn update(id: Uuid, user: UpdatedUser) -> Result<Self, CustomError> {
        let connection = &mut establish_connection();
        let updated_user = diesel::update(users::table)
            .filter(users::user_id.eq(id))
            .set(user)
            .get_result(connection)?;
        Ok(updated_user)
    }

    pub fn delete(user_id: Uuid) -> Result<usize, CustomError> {
        let connection = &mut establish_connection();
        let res =
            diesel::delete(users::table.filter(users::user_id.eq(user_id))).execute(connection)?;
        Ok(res)
    }

    pub fn hash_password(plain: String) -> Result<String, CustomError> {
        Ok(hash(plain, DEFAULT_COST)?)
    }

    pub fn find_any() -> Result<bool, CustomError> {
        let connection = &mut establish_connection();
        let res = dsl::select(dsl::exists(users::table.select(users::user_id).limit(1)))
            .get_result(connection)?;
        Ok(res)
    }
}

#[derive(Deserialize)]
pub struct AuthUser {
    pub email: String,
    pub password: String,
}

impl AuthUser {
    pub fn login(&self) -> Result<User, AuthError> {
        let connection = &mut establish_connection();
        let user: User = users::table
            .filter(users::email.eq(&self.email))
            .first(connection)?;

        let Some(password) = &user.password else {
            return Err(AuthError::WrongPassword("Invalid account".to_string()));
        };

        let verify_password = verify(&self.password, password).map_err(|_error| {
            AuthError::WrongPassword("Wrong password, check again please".to_string())
        })?;

        if verify_password {
            Ok(user)
        } else {
            Err(AuthError::WrongPassword(
                "Wrong password, check again please".to_string(),
            ))
        }
    }
}
