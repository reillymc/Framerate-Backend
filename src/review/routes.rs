use crate::error_handler::CustomError;
use crate::review_company::ReviewCompanyDetails;
use crate::user::placeholder_user;
use actix_web::{get, post, put, web, HttpResponse};
use chrono::NaiveDate;
use serde::Deserialize;
use serde::Serialize;
use uuid::Uuid;

use super::NewReview;
use super::Review;

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ReviewReadResponse {
    pub review_id: Uuid,
    pub user_id: Uuid,
    pub media_id: i32,
    pub imdb_id: Option<String>,
    pub media_type: String,
    pub media_title: String,
    pub media_poster_uri: Option<String>,
    pub media_release_year: i16,
    pub date: Option<NaiveDate>,
    pub rating: i16,
    pub review_title: Option<String>,
    pub review_description: Option<String>,
    pub venue: Option<String>,
    pub company: Option<Vec<ReviewCompanyDetails>>,
}

impl From<Review> for ReviewReadResponse {
    fn from(review: Review) -> Self {
        ReviewReadResponse {
            review_id: review.review_id,
            user_id: review.user_id,
            media_id: review.media_id,
            imdb_id: review.imdb_id,
            media_type: review.media_type,
            media_title: review.media_title,
            media_poster_uri: review.media_poster_uri,
            media_release_year: review.media_release_year,
            date: review.date,
            rating: review.rating,
            review_title: review.review_title,
            review_description: review.review_description,
            venue: review.venue,
            company: None,
        }
    }
}

#[get("/reviews/{review_id}")]
async fn find(review_id: web::Path<Uuid>) -> Result<HttpResponse, CustomError> {
    let review = Review::find(review_id.into_inner())?;
    let company = crate::review_company::ReviewCompany::find_by_review(review.review_id)?;
    Ok(HttpResponse::Ok().json({
        let mut review = ReviewReadResponse::from(review);
        review.company = Some(company.into_iter().map(|c| c.into()).collect());
        review
    }))
}

#[get("/reviews/media/{media_id}")]
async fn find_by_media(media_id: web::Path<i32>) -> Result<HttpResponse, CustomError> {
    let review = Review::find_by_media(media_id.into_inner())?;
    Ok(HttpResponse::Ok().json(review))
}

#[get("/reviews")]
async fn find_all() -> Result<HttpResponse, CustomError> {
    let user_id = placeholder_user();
    let reviews = Review::find_by_user(user_id)?;
    Ok(HttpResponse::Ok().json(reviews))
}

#[post("/reviews")]
async fn create(review: web::Json<NewReview>) -> Result<HttpResponse, CustomError> {
    let company = review.company.clone();
    let created_review = Review::create(review.into_inner())?;

    if company.is_none() {
        return Ok(HttpResponse::Ok().json(created_review));
    } else {
        let company = crate::review_company::ReviewCompany::replace(
            created_review.review_id,
            company.unwrap(),
        )?;
        return Ok(HttpResponse::Ok().json({
            let mut review = ReviewReadResponse::from(created_review);
            review.company = Some(company.into_iter().map(|c| c.into()).collect());
            review
        }));
    }
}

#[put("/reviews/{review_id}")]
async fn update(
    review: web::Json<NewReview>,
    review_id: web::Path<Uuid>,
) -> Result<HttpResponse, CustomError> {
    let company = review.company.clone();

    let updated_review = Review::update(review_id.into_inner(), review.into_inner())?;

    if company.is_none() {
        return Ok(HttpResponse::Ok().json(updated_review));
    } else {
        let company = crate::review_company::ReviewCompany::replace(
            updated_review.review_id,
            company.unwrap(),
        )?;
        return Ok(HttpResponse::Ok().json({
            let mut review = ReviewReadResponse::from(updated_review);
            review.company = Some(company.into_iter().map(|c| c.into()).collect());
            review
        }));
    }
}
