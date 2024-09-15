use std::env;

use actix_web::HttpResponse;
use chrono::{Duration, Local};
use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, Validation};
use serde::{Deserialize, Serialize};
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
            exp: (Local::now() + Duration::hours(24)).timestamp() as usize,
        }
    }
}

pub fn create_token(id: Uuid, email: &str) -> Result<String, HttpResponse> {
    let claims = UserSession::with_email(id, email);
    encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(get_secret().as_ref()),
    )
    .map_err(|e| HttpResponse::InternalServerError().json(e.to_string()))
}

pub fn decode_token(token: &str) -> Result<Auth, HttpResponse> {
    decode::<UserSession>(
        token,
        &DecodingKey::from_secret(get_secret().as_ref()),
        &Validation::default(),
    )
    .map(|data| data.claims.into())
    .map_err(|e| HttpResponse::Unauthorized().json(e.to_string()))
}

fn get_secret() -> String {
    return env::var("JWT_SECRET").unwrap();
}
