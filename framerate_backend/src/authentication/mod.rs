pub mod routes;

use std::future::{ready, Ready};

use crate::utils::{
    jwt::{decode_token, Auth},
    response_body::Error,
};
use actix_web::{
    dev::Payload, error::InternalError, http::header, FromRequest, HttpRequest, HttpResponse,
};
pub use routes::*;

impl FromRequest for Auth {
    type Error = InternalError<String>;

    type Future = Ready<Result<Self, Self::Error>>;

    fn from_request(req: &HttpRequest, _: &mut Payload) -> Self::Future {
        let access_token = req
            .headers()
            .get(header::AUTHORIZATION)
            .and_then(|value| value.to_str().ok())
            .and_then(|str| str.split(" ").nth(1));

        match access_token {
            Some(token) => {
                let user = decode_token(token);

                match user {
                    Ok(user) => ready(Ok(user)),

                    Err(_) => ready(Err(InternalError::from_response(
                        String::from("Invalid token"),
                        HttpResponse::Unauthorized().json(Error {
                            message: String::from("Invalid token"),
                        }),
                    ))),
                }
            }

            None => ready(Err(InternalError::from_response(
                String::from("No token provided"),
                HttpResponse::Unauthorized().json(Error {
                    message: String::from("No token provided"),
                }),
            ))),
        }
    }
}
