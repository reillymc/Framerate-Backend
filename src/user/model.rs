use crate::db::DbConnection;
use crate::schema::users;
use crate::utils::AppError;
use bcrypt::{hash, verify, DEFAULT_COST};
use chrono::NaiveDateTime;
use diesel::{dsl, prelude::*};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(PartialEq, PartialOrd, Serialize, Deserialize, Debug)]
pub enum PermissionLevel {
    Unknown = -100,
    NonAuthenticatable = -20,
    Registered = -10,
    GeneralUser = 10,
    AdminUser = 20,
}

impl From<i16> for PermissionLevel {
    fn from(permission_level: i16) -> Self {
        match permission_level {
            -20 => PermissionLevel::NonAuthenticatable,
            -10 => PermissionLevel::Registered,
            10 => PermissionLevel::GeneralUser,
            20 => PermissionLevel::AdminUser,
            _ => PermissionLevel::Unknown,
        }
    }
}

impl From<PermissionLevel> for i16 {
    fn from(permission_level: PermissionLevel) -> Self {
        match permission_level {
            PermissionLevel::NonAuthenticatable => -20,
            PermissionLevel::Registered => -10,
            PermissionLevel::GeneralUser => 10,
            PermissionLevel::AdminUser => 20,
            PermissionLevel::Unknown => -100,
        }
    }
}

impl PermissionLevel {
    pub fn is_at_least_registered(&self) -> bool {
        self >= &PermissionLevel::Registered
    }

    pub fn is_at_least_general(&self) -> bool {
        self >= &PermissionLevel::GeneralUser
    }

    pub fn is_at_least_admin(&self) -> bool {
        self >= &PermissionLevel::AdminUser
    }
}

#[derive(Serialize, Deserialize, Clone, Queryable, Selectable, Insertable)]
#[diesel(table_name = users)]
#[serde(rename_all = "camelCase")]
pub struct User {
    pub user_id: Uuid,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub email: Option<String>,
    #[serde(skip)]
    pub password: Option<String>,
    pub first_name: String,
    pub last_name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub avatar_uri: Option<String>,
    pub date_created: NaiveDateTime,
    /**
     * -20: Non-authenticatable user (review company without actual account)
     * -10: Registered, but not approved (if approvals required)
     * 10: Regular user
     * 20: Admin user
     */
    pub permission_level: i16,
    pub public: bool,
    pub configuration: serde_json::Value,
    pub created_by: Option<Uuid>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NewUser {
    pub email: String,
    pub password: String,
    pub first_name: String,
    pub last_name: String,
    pub avatar_uri: Option<String>,
    pub is_admin: Option<bool>,
    pub configuration: Option<serde_json::Value>,
}

#[derive(Debug, Deserialize, AsChangeset)]
#[diesel(table_name = users)]
#[serde(rename_all = "camelCase")]
pub struct UpdatedUser {
    pub first_name: Option<String>,
    pub last_name: Option<String>,
    pub configuration: Option<serde_json::Value>,
}

#[derive(Serialize, Debug, Queryable)]
#[diesel(table_name = users)]
#[serde(deny_unknown_fields)]
#[serde(rename_all = "camelCase")]
pub struct UserResponse {
    pub user_id: Uuid,
    pub first_name: String,
    pub last_name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub avatar_uri: Option<String>,
}

#[derive(Serialize, Debug, Queryable)]
#[diesel(table_name = users)]
#[serde(rename_all = "camelCase")]
pub struct UserFindResponse {
    pub user_id: Uuid,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub email: Option<String>,
    pub first_name: String,
    pub last_name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub avatar_uri: Option<String>,
    pub configuration: serde_json::Value,
}

impl From<NewUser> for User {
    fn from(user: NewUser) -> Self {
        User {
            user_id: Uuid::new_v4(),
            email: Some(user.email),
            password: None,
            first_name: user.first_name,
            last_name: user.last_name,
            date_created: chrono::Local::now().naive_local(),
            permission_level: if user.is_admin.unwrap_or(false) {
                i16::from(PermissionLevel::AdminUser)
            } else {
                i16::from(PermissionLevel::GeneralUser)
            },
            public: false,
            avatar_uri: user.avatar_uri,
            configuration: serde_json::json!({
                "people": [],
                "venues": [],
            }),
            created_by: None,
        }
    }
}

impl User {
    pub fn password(mut self, password: String) -> Self {
        self.password = Some(password);
        self
    }

    pub fn permission_level(mut self, permission_level: PermissionLevel) -> Self {
        self.permission_level = i16::from(permission_level);
        self
    }
}

impl User {
    pub fn find(conn: &mut DbConnection, user_id: Uuid) -> Result<UserFindResponse, AppError> {
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
            .first(conn)?;
        Ok(users)
    }

    pub fn find_summary(conn: &mut DbConnection, user_id: Uuid) -> Result<UserResponse, AppError> {
        let users = users::table
            .select((
                users::user_id,
                users::first_name,
                users::last_name,
                users::avatar_uri,
            ))
            .filter(users::user_id.eq(user_id))
            .first(conn)?;
        Ok(users)
    }

    pub fn find_all(conn: &mut DbConnection) -> Result<Vec<UserResponse>, AppError> {
        let users = users::table
            .select((
                users::user_id,
                users::first_name,
                users::last_name,
                users::avatar_uri,
            ))
            .load(conn)?;
        Ok(users)
    }

    pub fn create(conn: &mut DbConnection, user: NewUser) -> Result<Self, AppError> {
        let password = Self::hash_password(&user.password)?;

        let user_to_save = User::from(user).password(password);

        let new_user = diesel::insert_into(users::table)
            .values(user_to_save)
            .get_result(conn)?;
        Ok(new_user)
    }

    pub fn create_admin(conn: &mut DbConnection, user: NewUser) -> Result<Self, AppError> {
        let password = Self::hash_password(&user.password)?;

        let user_to_save = User::from(user)
            .password(password)
            .permission_level(PermissionLevel::AdminUser);

        let new_user = diesel::insert_into(users::table)
            .values(user_to_save)
            .get_result(conn)?;
        Ok(new_user)
    }

    pub fn update(
        conn: &mut DbConnection,
        user_id: Uuid,
        user: UpdatedUser,
    ) -> Result<Self, AppError> {
        let updated_user = diesel::update(users::table)
            .filter(users::user_id.eq(user_id))
            .set(user)
            .get_result(conn)?;
        Ok(updated_user)
    }

    pub fn delete(conn: &mut DbConnection, user_id: Uuid) -> Result<usize, AppError> {
        let res = diesel::delete(users::table.filter(users::user_id.eq(user_id))).execute(conn)?;
        Ok(res)
    }

    pub fn hash_password(plain: &str) -> Result<String, AppError> {
        Ok(hash(plain, DEFAULT_COST)?)
    }

    pub fn find_any(conn: &mut DbConnection) -> Result<bool, AppError> {
        let res = dsl::select(dsl::exists(users::table.select(users::user_id).limit(1)))
            .get_result(conn)?;
        Ok(res)
    }
}

#[derive(Deserialize)]
pub struct AuthUser {
    pub email: String,
    pub password: String,
}

impl AuthUser {
    pub fn login(&self, conn: &mut DbConnection) -> Result<User, AppError> {
        let user: Result<User, diesel::result::Error> = users::table
            .filter(users::email.eq(&self.email))
            .filter(users::permission_level.ge::<i16>(PermissionLevel::GeneralUser.into()))
            .first(conn);

        // Map 404 to 401 to avoid leaking emails in database
        let user = match user {
            Ok(user) => Ok(user),
            Err(err) => match err {
                diesel::result::Error::NotFound => {
                    Err(AppError::external(401, "Invalid email or password"))
                }
                _ => Err(AppError::DieselError(err)),
            },
        }?;

        let Some(password) = &user.password else {
            return Err(AppError::external(401, "Invalid email or password"));
        };

        let Ok(verify_password) = verify(&self.password, password) else {
            return Err(AppError::external(401, "Invalid email or password"));
        };

        if verify_password {
            Ok(user)
        } else {
            Err(AppError::external(401, "Invalid email or password"))
        }
    }
}
