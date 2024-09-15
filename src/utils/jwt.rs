use crate::error_handler::CustomError;
use chrono::{Duration, Local};
use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, Validation};
use serde::{Deserialize, Serialize};
use std::env;
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize)]
struct UserSession {
    id: Uuid,
    email: String,
    exp: usize,
}

pub struct Auth {
    pub user_id: Uuid,
}

impl From<UserSession> for Auth {
    fn from(user_session: UserSession) -> Self {
        Auth {
            user_id: user_session.id,
        }
    }
}

impl UserSession {
    fn with_email(id: Uuid, email: &str) -> Self {
        UserSession {
            id,
            email: email.into(),
            // TODO: investigate refresh tokens
            exp: (Local::now() + Duration::weeks(52)).timestamp() as usize,
        }
    }
}

pub fn create_token(id: Uuid, email: &str) -> Result<String, CustomError> {
    let claims = UserSession::with_email(id, email);
    let encoded = encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(get_secret().as_ref()),
    )?;

    Ok(encoded)
}

pub fn decode_token(token: &str) -> Result<Auth, CustomError> {
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
