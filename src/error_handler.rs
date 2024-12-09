use actix_web::{http::StatusCode, HttpResponse, ResponseError};
use bcrypt::BcryptError;
use diesel::result::{self, Error as DieselError};
use r2d2::Error as R2D2Error;
use serde::Deserialize;
use serde_json::json;
use std::fmt;

pub enum AuthError {
    // HashError(BcryptError),
    // PasswordNotMatch(String),
    WrongPassword(String),
    DBError(result::Error),
}

impl From<result::Error> for AuthError {
    fn from(error: result::Error) -> Self {
        AuthError::DBError(error)
    }
}

impl fmt::Display for AuthError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            // MyStoreError::HashError(error) => write!(f, "{}", error),
            AuthError::DBError(error) => write!(f, "{}", error),
            // MyStoreError::PasswordNotMatch(error) => write!(f, "{}", error),
            AuthError::WrongPassword(error) => write!(f, "{}", error),
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct CustomError {
    pub status_code: u16,
    pub message: String,
}

impl CustomError {
    pub fn new(error_status_code: u16, error_message: &str) -> CustomError {
        CustomError {
            status_code: error_status_code,
            message: error_message.to_string(),
        }
    }
}

impl fmt::Display for CustomError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.write_str(self.message.as_str())
    }
}

impl From<DieselError> for CustomError {
    fn from(error: DieselError) -> CustomError {
        match error {
            DieselError::DatabaseError(_, err) => CustomError::new(409, err.message()),
            DieselError::NotFound => CustomError::new(404, "Not found"),
            _err => CustomError::new(500, "Unknown Internal Error"),
        }
    }
}

impl From<reqwest::Error> for CustomError {
    fn from(_error: reqwest::Error) -> CustomError {
        CustomError::new(500, "Unable to complete external request")
    }
}

impl From<reqwest_middleware::Error> for CustomError {
    fn from(_error: reqwest_middleware::Error) -> CustomError {
        CustomError::new(500, "Unable to complete external request")
    }
}

impl From<BcryptError> for CustomError {
    fn from(_error: BcryptError) -> CustomError {
        CustomError::new(500, "Internal error")
    }
}

impl From<jsonwebtoken::errors::Error> for CustomError {
    fn from(_error: jsonwebtoken::errors::Error) -> CustomError {
        CustomError::new(500, "Auth error")
    }
}

impl From<R2D2Error> for CustomError {
    fn from(_error: R2D2Error) -> CustomError {
        CustomError::new(500, "Internal service error")
    }
}

impl ResponseError for CustomError {
    fn error_response(&self) -> HttpResponse {
        let status_code = match StatusCode::from_u16(self.status_code) {
            Ok(status_code) => status_code,
            Err(_) => StatusCode::INTERNAL_SERVER_ERROR,
        };

        HttpResponse::build(status_code).json(json!({ "message": self.message }))
    }
}
