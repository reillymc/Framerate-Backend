use crate::error_handler::CustomError;
use actix_web::{get, post, web, HttpResponse};
use uuid::Uuid;

use super::User;

#[get("/user/{user_id}")]
async fn find(user_id: web::Path<Uuid>) -> Result<HttpResponse, CustomError> {
    let user = User::find(user_id.into_inner());

    match user {
        Ok(_) => Ok(HttpResponse::Ok().json(user.unwrap())),
        Err(_) => Err(CustomError::new(404, "User not found".to_string())),
    }
}

#[post("/user")]
async fn create(employee: web::Json<User>) -> Result<HttpResponse, CustomError> {
    let employee = User::create(employee.into_inner())?;
    Ok(HttpResponse::Ok().json(employee))
}
