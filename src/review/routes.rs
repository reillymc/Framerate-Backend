use crate::error_handler::CustomError;
use actix_web::{get, post, web, HttpResponse};
use uuid::Uuid;

use super::NewReview;
use super::Review;

#[get("/reviews/{review_id}")]
async fn find(review_id: web::Path<Uuid>) -> Result<HttpResponse, CustomError> {
    let review = Review::find(review_id.into_inner())?;
    Ok(HttpResponse::Ok().json(review))
}

#[get("/reviews/media/{media_id}")]
async fn find_by_media(media_id: web::Path<i32>) -> Result<HttpResponse, CustomError> {
    let review = Review::find_by_media(media_id.into_inner())?;
    Ok(HttpResponse::Ok().json(review))
}

#[get("/reviews")]
async fn find_all() -> Result<HttpResponse, CustomError> {
    let reviews = Review::find_all()?;
    Ok(HttpResponse::Ok().json(reviews))
}

#[post("/reviews")]
async fn create(review: web::Json<NewReview>) -> Result<HttpResponse, CustomError> {
    let review = Review::create(review.into_inner())?;
    Ok(HttpResponse::Ok().json(review))
}
