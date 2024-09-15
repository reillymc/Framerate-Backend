use crate::review::ReviewFindParameters;
use crate::review_company::{ReviewCompanyDetails, ReviewCompanySummary};
use crate::utils::jwt::Auth;
use crate::utils::response_body::{Error, Success, SuccessWithMessage};
use actix_web::Responder;
use actix_web::{get, post, put, web, HttpResponse};
use chrono::NaiveDate;
use serde::Deserialize;
use serde::Serialize;
use uuid::Uuid;

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
    pub media_id: i32,
    pub media_type: String,
    pub date: Option<NaiveDate>,
    pub rating: i16,
    pub review_title: Option<String>,
    pub review_description: Option<String>,
    pub venue: Option<String>,
    pub company: Option<Vec<ReviewCompanySummary>>,
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateReviewRequest {
    pub date: Option<NaiveDate>,
    pub rating: i16,
    pub review_title: Option<String>,
    pub review_description: Option<String>,
    pub venue: Option<String>,
    pub company: Option<Vec<ReviewCompanySummary>>,
}

#[get("/reviews/review/{review_id}")]
async fn find(auth: Auth, review_id: web::Path<Uuid>) -> impl Responder {
    let Ok(review) = Review::find(auth.user_id, review_id.into_inner()) else {
        return HttpResponse::NotFound().json(Error {
            message: "Review not found".to_string(),
        });
    };

    let company = crate::review_company::ReviewCompany::find_by_review(review.review_id);

    let Ok(company) = company else {
        return HttpResponse::Ok().json(SuccessWithMessage {
            data: ReviewReadResponse::from(review),
            message: "Company could not be retrieved".to_string(),
        });
    };

    let mut review = ReviewReadResponse::from(review);
    review.company = Some(company);

    HttpResponse::Ok().json(Success { data: review })
}

#[get("/reviews/media/{media_id}")]
async fn find_by_media(auth: Auth, media_id: web::Path<i32>) -> impl Responder {
    match Review::find_by_media(auth.user_id, media_id.into_inner()) {
        Err(err) => HttpResponse::InternalServerError().json(Error {
            message: err.message,
        }),
        Ok(review) => HttpResponse::Ok().json(Success { data: review }),
    }
}

#[get("/reviews")]
async fn find_all(auth: Auth, params: web::Query<ReviewFindParameters>) -> impl Responder {
    match Review::find_by_user(auth.user_id, params.into_inner()) {
        Err(err) => HttpResponse::InternalServerError().json(Error {
            message: err.message,
        }),
        Ok(reviews) => HttpResponse::Ok().json(Success { data: reviews }),
    }
}

#[post("/reviews")]
async fn create(auth: Auth, review: web::Json<SaveReviewRequest>) -> impl Responder {
    let Ok(movie) = crate::movie::Movie::find(review.media_id).await else {
        return HttpResponse::NotFound().json(Error {
            message: "Movie not found".to_string(),
        });
    };

    let review_to_save = Review {
        review_id: Uuid::new_v4(),
        user_id: auth.user_id,
        media_id: review.media_id,
        imdb_id: movie.imdb_id,
        media_type: review.media_type.clone(),
        media_title: movie.title,
        media_poster_uri: movie.poster_path,
        media_release_date: movie.release_date,
        date: review.date,
        rating: review.rating,
        review_title: review.review_title.clone(),
        review_description: review.review_description.clone(),
        venue: review.venue.clone(),
    };

    let Ok(created_review) = Review::create(review_to_save) else {
        return HttpResponse::InternalServerError().json(Error {
            message: "Review could not be created".to_string(),
        });
    };

    let Some(review_company) = review.company.clone() else {
        return HttpResponse::Ok().json(Success {
            data: created_review,
        });
    };

    let company =
        crate::review_company::ReviewCompany::replace(created_review.review_id, review_company);

    let Ok(company) = company else {
        return HttpResponse::Ok().json(SuccessWithMessage {
            data: ReviewReadResponse::from(created_review),
            message: "Company could not be created".to_string(),
        });
    };

    let mut review = ReviewReadResponse::from(created_review);
    review.company = Some(company);

    HttpResponse::Ok().json(Success { data: review })
}

#[put("/reviews/review/{review_id}")]
async fn update(
    auth: Auth,
    review: web::Json<UpdateReviewRequest>,
    review_id: web::Path<Uuid>,
) -> impl Responder {
    let Ok(existing_review) = Review::find(auth.user_id, review_id.clone()) else {
        return HttpResponse::NotFound().json(Error {
            message: "Review not found".to_string(),
        });
    };

    let Ok(movie) = crate::movie::Movie::find(existing_review.media_id).await else {
        return HttpResponse::NotFound().json(Error {
            message: "Movie not found".to_string(),
        });
    };

    let review_to_save = Review {
        review_id: existing_review.review_id,
        user_id: existing_review.user_id,
        media_id: existing_review.media_id,
        imdb_id: movie.imdb_id,
        media_type: existing_review.media_type,
        media_title: movie.title,
        media_poster_uri: movie.poster_path,
        media_release_date: movie.release_date,
        date: review.date,
        rating: review.rating,
        review_title: review.review_title.clone(),
        review_description: review.review_description.clone(),
        venue: review.venue.clone(),
    };

    let Ok(updated_review) = Review::update(review_to_save) else {
        return HttpResponse::InternalServerError().json(Error {
            message: "Review could not be updated".to_string(),
        });
    };

    let Some(review_company) = review.company.clone() else {
        return HttpResponse::Ok().json(Success {
            data: updated_review,
        });
    };

    let company =
        crate::review_company::ReviewCompany::replace(updated_review.review_id, review_company);

    let Ok(company) = company else {
        return HttpResponse::Ok().json(SuccessWithMessage {
            data: ReviewReadResponse::from(updated_review),
            message: "Company could not be updated".to_string(),
        });
    };

    let mut review = ReviewReadResponse::from(updated_review);
    review.company = Some(company);

    HttpResponse::Ok().json(Success { data: review })
}

#[get("/reviews/statistics")]
async fn find_statistics(auth: Auth) -> impl Responder {
    match Review::find_statistics(auth.user_id) {
        Err(err) => HttpResponse::InternalServerError().json(Error {
            message: err.message,
        }),
        Ok(stats) => HttpResponse::Ok().json(Success { data: stats }),
    }
}
