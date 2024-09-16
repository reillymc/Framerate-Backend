use std::env;

use crate::{
    user::{AuthUser, User},
    utils::{
        jwt::create_token,
        response_body::{Error, Success},
    },
};
use actix_web::{post, web, HttpResponse, Responder};
use serde::Deserialize;
use uuid::Uuid;

#[derive(Deserialize)]
struct Secret {
    secret: String,
}

#[post("/auth/login")]
pub async fn login(auth_user: web::Json<AuthUser>) -> impl Responder {
    let user = auth_user.login();

    let Ok(user_details) = user else {
        return HttpResponse::BadRequest().json(Error {
            message: "Invalid credentials".to_string(),
        });
    };

    let token = create_token(user_details.user_id, &user_details.email);

    let Ok(token_string) = token else {
        return HttpResponse::InternalServerError().json(Error {
            message: "Unable to create token".to_string(),
        });
    };
    HttpResponse::Ok().json(Success { data: token_string })
}

#[post("/auth/setup")]
pub async fn setup(secret: web::Json<Secret>) -> impl Responder {
    let Ok(setup_secret) = env::var("SETUP_SECRET") else {
        return HttpResponse::InternalServerError().json(Error {
            message: "Unable to run setup procedure".to_string(),
        });
    };

    if secret.secret != setup_secret {
        return HttpResponse::Forbidden().json(Error {
            message: "Unauthorized to run setup procedure".to_string(),
        });
    }

    let Ok(any_users) = User::find_any() else {
        return HttpResponse::InternalServerError().json(Error {
            message: "Unknown Error".to_string(),
        });
    };

    if any_users {
        return HttpResponse::Forbidden().json(Error {
            message: "Setup procedure already run".to_string(),
        });
    };

    let token = create_token(Uuid::default(), "");

    let Ok(token_string) = token else {
        return HttpResponse::InternalServerError().json(Error {
            message: "Unable to create token".to_string(),
        });
    };
    HttpResponse::Ok().json(Success { data: token_string })
}
