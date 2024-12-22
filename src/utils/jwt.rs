use crate::{user::PermissionLevel, utils::AppError};
use chrono::{Duration, Local};
use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, Validation};
use serde::{Deserialize, Serialize};
use std::env;
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize)]
struct UserSession {
    id: Uuid,
    permission_level: PermissionLevel,
    exp: usize,
}

pub struct Auth {
    pub user_id: Uuid,
    pub permission_level: PermissionLevel,
}

impl Auth {
    pub fn is_at_least_registered(&self) -> bool {
        self.permission_level >= PermissionLevel::Registered
    }

    pub fn is_at_least_admin(&self) -> bool {
        self.permission_level >= PermissionLevel::AdminUser
    }
}

impl From<UserSession> for Auth {
    fn from(user_session: UserSession) -> Self {
        Auth {
            user_id: user_session.id,
            permission_level: user_session.permission_level,
        }
    }
}

impl UserSession {
    fn new(id: Uuid, permission_level: PermissionLevel) -> Self {
        UserSession {
            id,
            permission_level,
            // TODO: investigate refresh tokens
            exp: (Local::now() + Duration::weeks(52)).timestamp() as usize,
        }
    }
}

pub fn create_token(id: Uuid, permission_level: PermissionLevel) -> Result<String, AppError> {
    let claims = UserSession::new(id, permission_level);
    let encoded = encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(get_secret().as_ref()),
    )?;

    Ok(encoded)
}

pub fn decode_token(token: &str) -> Result<Auth, AppError> {
    let decoded = decode::<UserSession>(
        token,
        &DecodingKey::from_secret(get_secret().as_ref()),
        &Validation::default(),
    )
    .map(|data| data.claims.into())?;

    Ok(decoded)
}

fn get_secret() -> String {
    env::var("JWT_SECRET").unwrap()
}
