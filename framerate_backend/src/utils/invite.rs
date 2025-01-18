use crate::utils::AppError;
use chrono::{Duration, Local};
use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, Validation};
use serde::{Deserialize, Serialize};
use std::env;

#[derive(Debug, Serialize, Deserialize)]
pub struct InviteCode {
    pub email: String,
    exp: usize,
}

pub struct Invite {
    pub email: String,
}

impl InviteCode {
    fn new(email: String, expiry_time: chrono::DateTime<Local>) -> Self {
        InviteCode {
            email,
            exp: expiry_time.timestamp() as usize,
        }
    }
}

pub fn create_invite(email: String) -> Result<String, AppError> {
    let claims = InviteCode::new(email, Local::now() + Duration::days(1));
    let encoded = encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(get_invite_secret().as_ref()),
    )?;

    Ok(encoded)
}

pub fn decode_invite(token: &str) -> Result<InviteCode, AppError> {
    let decoded = decode::<InviteCode>(
        token,
        &DecodingKey::from_secret(get_invite_secret().as_ref()),
        &Validation::default(),
    )
    .map(|data| data.claims)?;

    Ok(decoded)
}

fn get_invite_secret() -> String {
    env::var("INVITE_SECRET").unwrap() + "_invite"
}
