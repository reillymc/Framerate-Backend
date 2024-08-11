use crate::{error_handler::CustomError, user::NewUser};
use actix_web::{get, post, put, web, HttpResponse};
use uuid::Uuid;

use super::{UpdatedUser, User};

#[get("/users")]
async fn find_all() -> Result<HttpResponse, CustomError> {
    let users = User::find_all()?;
    Ok(HttpResponse::Ok().json(users))
}

#[get("/users/{user_id}")]
async fn find(user_id: web::Path<Uuid>) -> Result<HttpResponse, CustomError> {
    let user = User::find(user_id.into_inner());

    match user {
        Ok(_) => Ok(HttpResponse::Ok().json(user.unwrap())),
        Err(_) => Err(CustomError::new(404, "User not found".to_string())),
    }
}

#[post("/users")]
async fn create(user: web::Json<NewUser>) -> Result<HttpResponse, CustomError> {
    let user = User::create(user.into_inner())?;
    Ok(HttpResponse::Ok().json(user))
}

#[put("/users/{user_id}")]
async fn update(
    user_id: web::Path<Uuid>,
    user: web::Json<UpdatedUser>,
) -> Result<HttpResponse, CustomError> {
    let user = User::update(user_id.into_inner(), user.into_inner());

    match user {
        Ok(_) => Ok(HttpResponse::Ok().json(user.unwrap())),
        Err(_) => Err(CustomError::new(404, "User not found".to_string())),
    }
}
