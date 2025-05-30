use crate::db::DbConnection;
use crate::schema::users;
use crate::utils::AppError;
use bcrypt::{hash, verify, DEFAULT_COST};
use chrono::NaiveDateTime;
use diesel::{dsl, prelude::*};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
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

#[derive(Serialize, Deserialize, Clone, Queryable, Selectable, Insertable, ToSchema)]
#[diesel(table_name = users)]
#[serde(rename_all = "camelCase")]
pub struct User {
    pub user_id: Uuid,
    pub email: String,
    #[serde(skip)]
    pub password: String,
    pub first_name: String,
    pub last_name: String,
    #[schema(nullable = false)]
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
    #[schema(nullable = false)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub created_by: Option<Uuid>,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct RegisteringUser {
    pub email: String,
    pub password: String,
    pub first_name: String,
    pub last_name: String,
    #[schema(nullable = false)]
    pub invite_code: Option<String>,
}

impl RegisteringUser {
    pub fn invite_code(mut self, invite_code: String) -> Self {
        self.invite_code = Some(invite_code);
        self
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct NewUser {
    pub email: String,
    pub password: String,
    pub first_name: String,
    pub last_name: String,
    #[schema(nullable = false)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub avatar_uri: Option<String>,
    #[schema(nullable = false)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub is_admin: Option<bool>,
    pub configuration: Option<serde_json::Value>,
}

#[derive(Debug, Deserialize, AsChangeset, ToSchema)]
#[diesel(table_name = users)]
#[serde(rename_all = "camelCase")]
pub struct UpdatedUser {
    #[schema(nullable = false)]
    pub first_name: Option<String>,
    #[schema(nullable = false)]
    pub last_name: Option<String>,
    pub configuration: Option<serde_json::Value>,
}

#[derive(Serialize, Debug, Queryable, ToSchema)]
#[diesel(table_name = users)]
#[serde(deny_unknown_fields)]
#[serde(rename_all = "camelCase")]
pub struct UserResponse {
    pub user_id: Uuid,
    pub first_name: String,
    pub last_name: String,
    #[schema(nullable = false)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub avatar_uri: Option<String>,
}

#[derive(Serialize, Debug, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct UserFindResponse {
    pub user_id: Uuid,
    pub email: String,
    pub first_name: String,
    pub last_name: String,
    #[schema(nullable = false)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub avatar_uri: Option<String>,
    pub configuration: serde_json::Value,
    pub is_admin: bool,
}

#[derive(Debug, Queryable)]
#[diesel(table_name = users)]
pub struct InternalUserFindResponse {
    pub user_id: Uuid,
    pub email: String,
    pub first_name: String,
    pub last_name: String,
    pub avatar_uri: Option<String>,
    pub configuration: serde_json::Value,
    pub permission_level: i16,
}

impl From<InternalUserFindResponse> for UserFindResponse {
    fn from(value: InternalUserFindResponse) -> Self {
        UserFindResponse {
            user_id: value.user_id,
            email: value.email,
            first_name: value.first_name,
            last_name: value.last_name,
            is_admin: PermissionLevel::from(value.permission_level).is_at_least_admin(),
            avatar_uri: value.avatar_uri,
            configuration: value.configuration,
        }
    }
}

impl TryFrom<NewUser> for User {
    type Error = AppError;

    fn try_from(value: NewUser) -> Result<Self, AppError> {
        let password = User::hash_password(&value.password)?;

        Ok(User {
            user_id: Uuid::new_v4(),
            email: value.email,
            password,
            first_name: value.first_name,
            last_name: value.last_name,
            date_created: chrono::Local::now().naive_local(),
            permission_level: if value.is_admin.unwrap_or(false) {
                i16::from(PermissionLevel::AdminUser)
            } else {
                i16::from(PermissionLevel::GeneralUser)
            },
            public: false,
            avatar_uri: value.avatar_uri,
            configuration: serde_json::json!({
                "people": [],
                "venues": [],
            }),
            created_by: None,
        })
    }
}

impl User {
    pub fn password(mut self, password: String) -> Self {
        self.password = password;
        self
    }

    pub fn permission_level(mut self, permission_level: PermissionLevel) -> Self {
        self.permission_level = i16::from(permission_level);
        self
    }
}

impl User {
    pub fn find(conn: &mut DbConnection, user_id: Uuid) -> Result<UserFindResponse, AppError> {
        let user: InternalUserFindResponse = users::table
            .select((
                users::user_id,
                users::email,
                users::first_name,
                users::last_name,
                users::avatar_uri,
                users::configuration,
                users::permission_level,
            ))
            .filter(users::user_id.eq(user_id))
            .first(conn)?;
        Ok(user.into())
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
        let user_to_save = User::try_from(user)?;

        let new_user = diesel::insert_into(users::table)
            .values(user_to_save)
            .get_result(conn)?;
        Ok(new_user)
    }

    pub fn create_admin(conn: &mut DbConnection, user: NewUser) -> Result<Self, AppError> {
        let user_to_save = User::try_from(user)?.permission_level(PermissionLevel::AdminUser);

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

#[derive(Deserialize, ToSchema)]
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

        let Ok(verify_password) = verify(&self.password, &user.password) else {
            return Err(AppError::external(401, "Invalid email or password"));
        };

        if verify_password {
            Ok(user)
        } else {
            Err(AppError::external(401, "Invalid email or password"))
        }
    }
}
