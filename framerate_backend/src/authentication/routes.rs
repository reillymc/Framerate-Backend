use std::env;

use crate::{
    db::DbPool,
    user::{AuthUser, NewUser, PermissionLevel, RegisteringUser, User},
    utils::{invite::decode_invite, jwt::create_token, response_body::Success, AppError},
};
use actix_web::{post, web, Responder};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::Uuid;

#[derive(Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct LoginResponse {
    pub token: String,
    pub user_id: Uuid,
}

#[utoipa::path(tag = "Authentication", responses((status = OK, body = LoginResponse),(status = BAD_REQUEST),(status = UNAUTHORIZED)))]
#[post("/authentication/login")]
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

    if !permission_level.is_at_least_general() {
        return Err(AppError::external(401, "Invalid account"))?;
    };

    let token = create_token(user_details.user_id, permission_level)?;

    Ok(Success::new(LoginResponse {
        user_id: user_details.user_id,
        token,
    }))
}

#[utoipa::path(tag = "Authentication", responses((status = OK, body = LoginResponse),(status = BAD_REQUEST),(status = UNAUTHORIZED)))]
#[post("/authentication/register")]
pub async fn register(
    pool: web::Data<DbPool>,
    registering_user: web::Json<RegisteringUser>,
) -> actix_web::Result<impl Responder> {
    let Ok(registration_mode) = env::var("REGISTRATION_MODE") else {
        return Err(AppError::external(
            401,
            "Registrations are not currently open",
        ))?;
    };

    if !(registration_mode == "open".to_string() || registration_mode == "invite".to_string()) {
        return Err(AppError::external(
            401,
            "Registrations are not currently open",
        ))?;
    }

    let registering_user = registering_user.into_inner();

    if registering_user.email.is_empty() || registering_user.password.is_empty() {
        return Err(AppError::external(400, "Email and password are required"))?;
    }

    if registration_mode == "invite".to_string() {
        let Some(invite_code) = &registering_user.invite_code else {
            return Err(AppError::external(
                401,
                "You do not have a valid invite code",
            ))?;
        };

        let invite_details = decode_invite(invite_code)?;

        if invite_details.email != registering_user.email {
            return Err(AppError::external(
                401,
                "You do not have a valid invite code",
            ))?;
        };
    };

    let user = web::block(move || {
        let mut conn = pool.get()?;
        User::create(
            &mut conn,
            NewUser {
                avatar_uri: None,
                configuration: None,
                email: registering_user.email,
                first_name: registering_user.first_name,
                last_name: registering_user.last_name,
                password: registering_user.password,
                is_admin: Some(false),
            },
        )
    })
    .await??;

    let token = create_token(user.user_id, user.permission_level.into())?;

    Ok(Success::new(LoginResponse {
        user_id: user.user_id,
        token,
    }))
}
