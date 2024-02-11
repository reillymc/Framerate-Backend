use crate::error_handler::CustomError;
use actix_web::{get, post, web, HttpResponse};
use uuid::Uuid;

use super::Rating;

#[get("/rating/{rating_id}")]
async fn find(rating_id: web::Path<Uuid>) -> Result<HttpResponse, CustomError> {
    let rating = Rating::find(rating_id.into_inner())?;
    Ok(HttpResponse::Ok().json(rating))
}

#[post("/rating")]
async fn create(employee: web::Json<Rating>) -> Result<HttpResponse, CustomError> {
    let employee = Rating::create(employee.into_inner())?;
    Ok(HttpResponse::Ok().json(employee))
}
