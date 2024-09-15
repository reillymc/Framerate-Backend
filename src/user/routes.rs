use crate::{
    user::NewUser,
    utils::{
        jwt::Auth,
        response_body::{Error, Success},
    },
};
use actix_web::{get, post, put, web, HttpResponse, Responder};
use uuid::Uuid;

use super::{UpdatedUser, User};

#[get("/users")]
async fn find_all(_: Auth) -> impl Responder {
    let Ok(users) = User::find_all() else {
        return HttpResponse::InternalServerError().json(Error {
            message: "Internal Server Error".to_string(),
        });
    };

    HttpResponse::Ok().json(Success { data: users })
}

#[get("/users/{user_id}")]
async fn find(auth: Auth, user_id: web::Path<Uuid>) -> impl Responder {
    if auth.user_id == user_id.clone() {
        let Ok(user) = User::find(user_id.into_inner()) else {
            return HttpResponse::NotFound().json(Error {
                message: "User not found".to_string(),
            });
        };
        HttpResponse::Ok().json(Success { data: user })
    } else {
        let Ok(user) = User::find_summary(user_id.into_inner()) else {
            return HttpResponse::NotFound().json(Error {
                message: "User not found".to_string(),
            });
        };
        HttpResponse::Ok().json(Success { data: user })
    }
}

#[post("/users")]
async fn create(_: Auth, user: web::Json<NewUser>) -> impl Responder {
    let Ok(user) = User::create(user.into_inner()) else {
        return HttpResponse::BadRequest().json(Error {
            message: "Invalid data".to_string(),
        });
    };

    HttpResponse::Ok().json(Success { data: user })
}

#[put("/users/{user_id}")]
async fn update(
    auth: Auth,
    user_id: web::Path<Uuid>,
    user: web::Json<UpdatedUser>,
) -> impl Responder {
    if auth.user_id != user_id.clone() {
        return HttpResponse::Forbidden().json(Error {
            message: "Missing permission to update this user".to_string(),
        });
    }

    let Ok(user) = User::update(user_id.into_inner(), user.into_inner()) else {
        return HttpResponse::NotFound().json(Error {
            message: "User not found".to_string(),
        });
    };

    HttpResponse::Ok().json(Success { data: user })
}
