use std::env;

use crate::{
    db::DbPool,
    user::{AuthUser, PermissionLevel, User},
    utils::{jwt::create_token, response_body::Success, AppError},
};
use actix_web::{post, web, Responder};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Deserialize)]
struct Secret {
    secret: String,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct LoginResponse {
    token: String,
    user_id: Uuid,
}

#[post("/auth/login")]
pub async fn login(
    pool: web::Data<DbPool>,
    auth_user: web::Json<AuthUser>,
) -> actix_web::Result<impl Responder> {
    if auth_user.email.is_empty() || auth_user.password.is_empty() {
        return Err(AppError::external(400, "Email and password are required"))?;
    }

    let user_details = web::block(move || {
        let mut conn = pool.get()?;
        auth_user.login(&mut conn)
    })
    .await??;

    let permission_level = PermissionLevel::from(user_details.permission_level);

    if !permission_level.is_at_least_general() || user_details.email.is_none() {
        return Err(AppError::external(401, "Invalid account"))?;
    };

    let token = create_token(user_details.user_id, permission_level)?;

    Ok(Success::new(LoginResponse {
        user_id: user_details.user_id,
        token,
    }))
}

#[post("/auth/setup")]
pub async fn setup(
    pool: web::Data<DbPool>,
    secret: web::Json<Secret>,
) -> actix_web::Result<impl Responder> {
    let Ok(setup_secret) = env::var("SETUP_SECRET") else {
        return Err(AppError::external(500, "Unable to run setup procedure"))?;
    };

    if secret.secret != setup_secret {
        return Err(AppError::external(
            401,
            "Unauthorized to run setup procedure",
        ))?;
    }

    let any_users = web::block(move || {
        let mut conn = pool.get()?;
        User::find_any(&mut conn)
    })
    .await??;

    if any_users {
        return Err(AppError::external(401, "Setup procedure already run"))?;
    };

    let token = create_token(Uuid::default(), PermissionLevel::AdminUser);

    let Ok(token_string) = token else {
        return Err(AppError::external(500, "Unable to create token"))?;
    };

    Ok(Success::new(token_string))
}
