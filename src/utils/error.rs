use actix_web::{http::StatusCode, HttpResponse, ResponseError};
use bcrypt::BcryptError;
use diesel::result::Error as DieselError;
use r2d2::Error as R2D2Error;
use serde::Deserialize;
use serde_json::json;
use std::fmt;
use tracing::warn;

#[derive(Debug, Deserialize)]
pub struct CustomError {
    pub status_code: u16,
    pub message: String,
}

#[derive(Debug)]
pub enum AppError {
    DieselError(DieselError),
    R2D2Error(R2D2Error),
    ReqwestError(reqwest::Error),
    ReqwestMiddlewareError(reqwest_middleware::Error),
    CustomInternal(CustomError),
    CustomExternal(CustomError),
    TmdbError(CustomError),
    BcryptError(BcryptError),
    JwtError(jsonwebtoken::errors::Error),
}

impl fmt::Display for AppError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            AppError::DieselError(e) => e.fmt(f),
            AppError::R2D2Error(e) => e.fmt(f),
            AppError::ReqwestError(e) => e.fmt(f),
            AppError::ReqwestMiddlewareError(e) => e.fmt(f),
            AppError::CustomInternal(e) => e.fmt(f),
            AppError::CustomExternal(e) => e.fmt(f),
            AppError::TmdbError(e) => e.fmt(f),
            AppError::BcryptError(e) => e.fmt(f),
            AppError::JwtError(e) => e.fmt(f),
        }
    }
}

impl From<DieselError> for AppError {
    fn from(error: DieselError) -> AppError {
        AppError::DieselError(error)
    }
}

impl From<R2D2Error> for AppError {
    fn from(error: R2D2Error) -> AppError {
        AppError::R2D2Error(error)
    }
}

impl From<reqwest::Error> for AppError {
    fn from(error: reqwest::Error) -> AppError {
        AppError::ReqwestError(error)
    }
}

impl From<reqwest_middleware::Error> for AppError {
    fn from(error: reqwest_middleware::Error) -> AppError {
        AppError::ReqwestMiddlewareError(error)
    }
}

impl From<BcryptError> for AppError {
    fn from(error: BcryptError) -> AppError {
        AppError::BcryptError(error)
    }
}

impl From<jsonwebtoken::errors::Error> for AppError {
    fn from(error: jsonwebtoken::errors::Error) -> AppError {
        AppError::JwtError(error)
    }
}

impl AppError {
    pub fn tmdb_error(error_status_code: u16, error_message: &str) -> AppError {
        AppError::TmdbError(CustomError::new(error_status_code, error_message))
    }

    pub fn external(error_status_code: u16, error_message: &str) -> AppError {
        AppError::CustomExternal(CustomError::new(error_status_code, error_message))
    }
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

impl ResponseError for AppError {
    fn error_response(&self) -> HttpResponse {
        let status_code = match self {
            AppError::DieselError(error) => {
                warn!("Database Error: {error}");
                match error {
                    DieselError::NotFound => StatusCode::NOT_FOUND,
                    _ => StatusCode::INTERNAL_SERVER_ERROR,
                }
            }
            AppError::R2D2Error(error) => {
                warn!("DB Connection Error: {}", error);
                StatusCode::INTERNAL_SERVER_ERROR
            }
            AppError::ReqwestError(error) => {
                warn!("Reqwest Error: {}", error);
                StatusCode::INTERNAL_SERVER_ERROR
            }
            AppError::ReqwestMiddlewareError(error) => {
                warn!("Reqwest Error: {}", error);
                StatusCode::INTERNAL_SERVER_ERROR
            }
            AppError::TmdbError(error) => {
                warn!("TMDB Error: {}", error);
                StatusCode::from_u16(error.status_code).unwrap_or(StatusCode::INTERNAL_SERVER_ERROR)
            }
            AppError::CustomInternal(error) => {
                warn!("Custom Internal Error: {}", error);
                StatusCode::INTERNAL_SERVER_ERROR
            }
            AppError::CustomExternal(error) => {
                warn!("Custom External Error: {}", error);
                StatusCode::from_u16(error.status_code).unwrap_or(StatusCode::INTERNAL_SERVER_ERROR)
            }
            AppError::BcryptError(error) => {
                warn!("Bcrypt Error: {}", error);
                StatusCode::INTERNAL_SERVER_ERROR
            }
            AppError::JwtError(error) => {
                warn!("JWT Error: {}", error);
                StatusCode::INTERNAL_SERVER_ERROR
            }
        };

        let message = match self {
            AppError::CustomExternal(e) => e.message.as_str(),
            _ => match status_code {
                StatusCode::NOT_FOUND => "Item not found",
                _ => "Internal Server Error",
            },
        };

        HttpResponse::build(status_code).json(json!({ "message": message }))
    }
}
