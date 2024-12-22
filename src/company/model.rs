use crate::{
    db::DbConnection,
    utils::AppError,
    schema::users::{self},
    user::{PermissionLevel, User},
};
use chrono::NaiveDateTime;
use diesel::prelude::*;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Serialize, Deserialize, Selectable, Queryable)]
#[diesel(table_name = users)]
#[serde(rename_all = "camelCase")]
pub struct Company {
    pub user_id: Uuid,
    pub first_name: String,
    pub last_name: String,
    pub date_created: NaiveDateTime,
    pub created_by: Option<Uuid>,
}

#[derive(AsChangeset, Insertable, Serialize, Deserialize)]
#[diesel(table_name = users)]
#[serde(rename_all = "camelCase")]
pub struct SaveCompany {
    pub first_name: String,
    pub last_name: String,
}

impl From<SaveCompany> for User {
    fn from(user: SaveCompany) -> Self {
        User {
            user_id: Uuid::new_v4(),
            email: None,
            password: None,
            first_name: user.first_name,
            last_name: user.last_name,
            date_created: chrono::Local::now().naive_local(),
            permission_level: PermissionLevel::NonAuthenticatable.into(),
            public: false,
            avatar_uri: None,
            configuration: serde_json::json!({}),
            created_by: None,
        }
    }
}

impl User {
    fn created_by(mut self, created_by: Uuid) -> Self {
        self.created_by = Some(created_by);
        self
    }
}

impl From<User> for Company {
    fn from(user: User) -> Self {
        Company {
            user_id: user.user_id,
            created_by: user.created_by,
            date_created: user.date_created,
            first_name: user.first_name,
            last_name: user.last_name,
        }
    }
}

impl Company {
    pub fn find_all(conn: &mut DbConnection, created_by: &Uuid) -> Result<Vec<Self>, AppError> {
        let company = users::table
            .select((
                users::user_id,
                users::first_name,
                users::last_name,
                users::date_created,
                users::created_by,
            ))
            .filter(users::created_by.eq(created_by))
            .filter(users::permission_level.eq::<i16>(PermissionLevel::NonAuthenticatable.into()))
            .load(conn)?;
        Ok(company)
    }

    pub fn create(
        conn: &mut DbConnection,
        company: SaveCompany,
        created_by: Uuid,
    ) -> Result<Self, AppError> {
        let user_to_save = User::from(company).created_by(created_by);

        let new_user: User = diesel::insert_into(users::table)
            .values(user_to_save)
            .get_result(conn)?;

        Ok(new_user.into())
    }

    pub fn update(
        conn: &mut DbConnection,
        user_id: Uuid,
        company: SaveCompany,
        created_by: &Uuid,
    ) -> Result<Self, AppError> {
        let updated_user: User = diesel::update(users::table)
            .filter(users::created_by.eq(created_by))
            .filter(users::user_id.eq(user_id))
            .filter(users::permission_level.eq::<i16>(PermissionLevel::NonAuthenticatable.into()))
            .set(company)
            .get_result(conn)?;
        Ok(updated_user.into())
    }

    pub fn delete(
        conn: &mut DbConnection,
        user_id: Uuid,
        created_by: &Uuid,
    ) -> Result<usize, AppError> {
        let res = diesel::delete(users::table)
            .filter(users::created_by.eq(created_by))
            .filter(users::user_id.eq(user_id))
            .filter(users::permission_level.eq::<i16>(PermissionLevel::NonAuthenticatable.into()))
            .execute(conn)?;
        Ok(res)
    }
}
