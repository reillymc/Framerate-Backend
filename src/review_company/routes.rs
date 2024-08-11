use crate::error_handler::CustomError;
use actix_web::delete;
use actix_web::{get, post, web, HttpResponse};
use uuid::Uuid;

use super::ReviewCompany;

#[get("/reviews/{review_id}/company")]
async fn find_all(path: web::Path<Uuid>) -> Result<HttpResponse, CustomError> {
    let review_id = path.into_inner();
    let entries = ReviewCompany::find_all(review_id)?;
    Ok(HttpResponse::Ok().json(entries))
}

#[post("/reviews/{review_id}/company")]
async fn create(
    review_id: web::Path<Uuid>,
    review_company: web::Json<ReviewCompany>,
) -> Result<HttpResponse, CustomError> {
    let watchlist = ReviewCompany::create(review_id.into_inner(), review_company.into_inner())?;
    Ok(HttpResponse::Ok().json(watchlist))
}

#[delete("/reviews/{review_id}/company/{user_id}")]
async fn delete(path: web::Path<(Uuid, Uuid)>) -> Result<HttpResponse, CustomError> {
    let (review_id, user_id) = path.into_inner();
    let watchlist = ReviewCompany::delete(review_id, user_id)?;
    Ok(HttpResponse::Ok().json(watchlist))
}
