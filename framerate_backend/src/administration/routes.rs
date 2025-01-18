use std::env;

use crate::{
    db::DbPool,
    user::{PermissionLevel, User},
    utils::{
        invite::create_invite,
        jwt::{create_temp_token, Auth},
        response_body::Success,
        AppError,
    },
};
use actix_web::{post, web, Responder};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::Uuid;

#[derive(Deserialize, ToSchema)]
struct Secret {
    secret: String,
}

#[derive(Deserialize, Serialize, ToSchema)]
pub struct InviteDetails {
    pub email: String,
}

#[utoipa::path(tag = "Administration", responses((status = OK, body = String),(status = INTERNAL_SERVER_ERROR),(status = UNAUTHORIZED)))]
#[post("/administration/generate_setup_token")]
pub async fn generate_setup_token(
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

    let token = create_temp_token(Uuid::default(), PermissionLevel::AdminUser);

    let Ok(token_string) = token else {
        return Err(AppError::external(500, "Unable to create token"))?;
    };

    Ok(Success::new(token_string))
}

#[utoipa::path(tag = "Administration", responses((status = OK, body = String),(status = INTERNAL_SERVER_ERROR),(status = UNAUTHORIZED)))]
#[post("/administration/generate_invite")]
pub async fn generate_invite(
    auth: Auth,
    invite_details: web::Json<InviteDetails>,
) -> actix_web::Result<impl Responder> {
    let Ok(registration_mode) = env::var("REGISTRATION_MODE") else {
        return Err(AppError::external(
            401,
            "Registrations are not currently open",
        ))?;
    };

    if registration_mode != "invite".to_string() {
        return Err(AppError::external(
            500,
            "Unsupported registration configuration",
        ))?;
    }

    if !auth.is_at_least_admin() {
        return Err(AppError::external(401, "Unauthorized to invite users"))?;
    }

    if invite_details.email.is_empty() {
        return Err(AppError::external(400, "Invite email is mandatory"))?;
    }

    let invite_token = create_invite(invite_details.email.clone());

    let Ok(token_string) = invite_token else {
        return Err(AppError::external(500, "Unable to create invite"))?;
    };

    Ok(Success::new(token_string))
}
