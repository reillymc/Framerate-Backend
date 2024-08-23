use crate::error_handler::CustomError;
use crate::review::ReviewFindParameters;
use crate::review::UpdatedReview;
use crate::review_company::ReviewCompanyDetails;
use crate::review_company::ReviewCompanySummary;
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
    pub media_release_date: Option<NaiveDate>,
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
            media_release_date: review.media_release_date,
            date: review.date,
            rating: review.rating,
            review_title: review.review_title,
            review_description: review.review_description,
            venue: review.venue,
            company: None,
        }
    }
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SaveReviewRequest {
    pub review_id: Option<Uuid>,
    pub media_id: i32,
    pub media_type: String,
    pub date: Option<NaiveDate>,
    pub rating: i16,
    pub review_title: Option<String>,
    pub review_description: Option<String>,
    pub venue: Option<String>,
    pub company: Option<Vec<ReviewCompanySummary>>,
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
async fn find_all(params: web::Query<ReviewFindParameters>) -> Result<HttpResponse, CustomError> {
    let user_id = placeholder_user();
    let reviews = Review::find_by_user(user_id, params.into_inner())?;
    Ok(HttpResponse::Ok().json(reviews))
}

#[post("/reviews")]
async fn create(review: web::Json<SaveReviewRequest>) -> Result<HttpResponse, CustomError> {
    let movie = crate::movie::Movie::find(review.media_id).await;

    let movie_details = match movie {
        Ok(movie) => movie,
        Err(_) => {
            return Err(CustomError::new(
                404,
                "The requested movie was not found".to_string(),
            ))
        }
    };

    let review_to_save = NewReview {
        review_id: review.review_id,
        user_id: placeholder_user(),
        media_id: review.media_id,
        imdb_id: movie_details.imdb_id,
        media_type: review.media_type.clone(),
        media_title: movie_details.title,
        media_poster_uri: movie_details.poster_path,
        media_release_date: movie_details.release_date,
        date: review.date,
        rating: review.rating,
        review_title: review.review_title.clone(),
        review_description: review.review_description.clone(),
        venue: review.venue.clone(),
    };

    let created_review = Review::create(review_to_save)?;

    let review_company = match review.company.clone() {
        Some(review_company) => review_company,
        None => {
            return Ok(HttpResponse::Ok().json(created_review));
        }
    };

    let company =
        crate::review_company::ReviewCompany::replace(created_review.review_id, review_company)?;

    return Ok(HttpResponse::Ok().json({
        let mut review = ReviewReadResponse::from(created_review);
        review.company = Some(company.into_iter().map(|c| c.into()).collect());
        review
    }));
}

#[put("/reviews/{review_id}")]
async fn update(
    review: web::Json<SaveReviewRequest>,
    review_id: web::Path<Uuid>,
) -> Result<HttpResponse, CustomError> {
    let movie = crate::movie::Movie::find(review.media_id).await;

    let movie_details = match movie {
        Ok(movie) => movie,
        Err(_) => {
            return Err(CustomError::new(
                404,
                "The requested movie was not found".to_string(),
            ))
        }
    };

    let review_to_save = UpdatedReview {
        review_id: review_id.into_inner(),
        user_id: placeholder_user(),
        media_id: review.media_id,
        imdb_id: movie_details.imdb_id,
        media_type: review.media_type.clone(),
        media_title: movie_details.title,
        media_poster_uri: movie_details.poster_path,
        media_release_date: movie_details.release_date,
        date: review.date,
        rating: review.rating,
        review_title: review.review_title.clone(),
        review_description: review.review_description.clone(),
        venue: review.venue.clone(),
    };

    let updated_review = Review::update(review_to_save.review_id, review_to_save)?;

    let review_company = match review.company.clone() {
        Some(review_company) => review_company,
        None => {
            return Ok(HttpResponse::Ok().json(updated_review));
        }
    };

    let company =
        crate::review_company::ReviewCompany::replace(updated_review.review_id, review_company)?;

    return Ok(HttpResponse::Ok().json({
        let mut review = ReviewReadResponse::from(updated_review);
        review.company = Some(company.into_iter().map(|c| c.into()).collect());
        review
    }));
}
