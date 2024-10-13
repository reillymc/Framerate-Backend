use super::ShowReviewReadResponse;

use crate::review::{Review, ReviewFindParameters};
use crate::review_company::{ReviewCompanyDetails, ReviewCompanySummary};
use crate::show::Show;
use crate::show_review::ShowReview;
use crate::utils::jwt::Auth;
use crate::utils::response_body::{Error, Success, SuccessWithMessage};
use actix_web::{get, post, web, HttpResponse};
use actix_web::{put, Responder};
use chrono::NaiveDate;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SaveShowReviewRequest {
    pub rating: Option<i16>,
    pub date: Option<NaiveDate>,
    pub title: Option<String>,
    pub description: Option<String>,
    pub venue: Option<String>,
    pub company: Option<Vec<ReviewCompanySummary>>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ShowReviewResponse {
    pub review_id: Uuid,
    pub user_id: Uuid,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub date: Option<NaiveDate>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub rating: Option<i16>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub venue: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub company: Option<Vec<ReviewCompanyDetails>>,
    pub show: Show,
}

impl From<ShowReviewReadResponse> for ShowReviewResponse {
    fn from(review: ShowReviewReadResponse) -> Self {
        ShowReviewResponse {
            review_id: review.review_id,
            user_id: review.user_id,
            date: review.date,
            rating: review.rating,
            title: review.title,
            description: review.description,
            venue: review.venue,
            company: None,
            show: review.show,
        }
    }
}

#[get("/shows/reviews")]
async fn find_all(auth: Auth, params: web::Query<ReviewFindParameters>) -> impl Responder {
    match ShowReview::find_all_reviews(auth.user_id, params.into_inner()) {
        Err(err) => HttpResponse::InternalServerError().json(Error {
            message: err.message,
        }),
        Ok(reviews) => HttpResponse::Ok().json(Success {
            data: reviews
                .into_iter()
                .map(ShowReviewResponse::from)
                .collect::<Vec<ShowReviewResponse>>(),
        }),
    }
}

#[get("/shows/reviews/{review_id}")]
async fn find_by_review_id(auth: Auth, review_id: web::Path<Uuid>) -> impl Responder {
    let Ok(review) = ShowReview::find_by_review_id(auth.user_id, review_id.into_inner()) else {
        return HttpResponse::NotFound().json(Error {
            message: "Review not found".to_string(),
        });
    };

    let company = crate::review_company::ReviewCompany::find_by_review(review.review_id);

    let Ok(company) = company else {
        return HttpResponse::Ok().json(SuccessWithMessage {
            data: ShowReviewResponse::from(review),
            message: "Company could not be retrieved".to_string(),
        });
    };

    let mut review = ShowReviewResponse::from(review);
    review.company = Some(company);

    HttpResponse::Ok().json(Success { data: review })
}

#[get("/shows/{show_id}/reviews")]
async fn find_by_show_id(auth: Auth, show_id: web::Path<i32>) -> impl Responder {
    match ShowReview::find_by_show_id(auth.user_id, show_id.into_inner()) {
        Err(err) => HttpResponse::InternalServerError().json(Error {
            message: err.message,
        }),
        Ok(reviews) => HttpResponse::Ok().json(Success {
            data: reviews
                .into_iter()
                .map(ShowReviewResponse::from)
                .collect::<Vec<ShowReviewResponse>>(),
        }),
    }
}

#[post("/shows/{show_id}/reviews")]
async fn create(
    auth: Auth,
    review: web::Json<SaveShowReviewRequest>,
    show_id: web::Path<i32>,
) -> impl Responder {
    let review = review.into_inner();
    let show_id = show_id.into_inner();

    let Ok(show) = crate::show::Show::find(&show_id).await else {
        return HttpResponse::NotFound().json(Error {
            message: "Show not found".to_string(),
        });
    };

    let review_id = Uuid::new_v4();

    let review_to_save = Review {
        review_id,
        user_id: auth.user_id,
        date: review.date,
        rating: review.rating,
        title: review.title,
        description: review.description,
        venue: review.venue,
    };

    let Ok(created_review) = Review::create(review_to_save) else {
        return HttpResponse::InternalServerError().json(Error {
            message: "Review could not be created".to_string(),
        });
    };

    let imdb_id = if let Some(external_ids) = show.external_ids {
        external_ids.imdb_id
    } else {
        None
    };

    let show_review_to_save = ShowReview {
        review_id,
        show_id,
        imdb_id,
        user_id: auth.user_id,
        name: show.name,
        poster_path: show.poster_path,
        first_air_date: show.first_air_date,
    };

    let Ok(created_show_review) = ShowReview::create(show_review_to_save) else {
        return HttpResponse::InternalServerError().json(Error {
            message: "Show review could not be created".to_string(),
        });
    };

    let mut review_response = ShowReviewResponse {
        review_id: created_review.review_id,
        user_id: created_review.user_id,
        date: created_review.date,
        rating: created_review.rating,
        title: created_review.title,
        description: created_review.description,
        venue: created_review.venue,
        show: Show::from(created_show_review),
        company: None,
    };

    let Some(review_company) = review.company.clone() else {
        return HttpResponse::Ok().json(Success {
            data: review_response,
        });
    };

    let company =
        crate::review_company::ReviewCompany::replace(created_review.review_id, review_company);

    let Ok(company) = company else {
        return HttpResponse::Ok().json(SuccessWithMessage {
            data: review_response,
            message: "Company could not be created".to_string(),
        });
    };

    review_response.company = Some(company);

    HttpResponse::Ok().json(Success {
        data: review_response,
    })
}

#[put("/shows/{show_id}/reviews/{review_id}")]
async fn update(
    auth: Auth,
    review: web::Json<SaveShowReviewRequest>,
    path: web::Path<(i32, Uuid)>,
) -> impl Responder {
    let (_, review_id) = path.into_inner();

    let Ok(existing_review) = ShowReview::find_by_review_id(auth.user_id, review_id) else {
        return HttpResponse::NotFound().json(Error {
            message: "Review not found".to_string(),
        });
    };

    let Ok(show) = crate::show::Show::find(&existing_review.show.id).await else {
        return HttpResponse::NotFound().json(Error {
            message: "Show not found".to_string(),
        });
    };

    let review_to_save = Review {
        review_id: existing_review.review_id,
        user_id: existing_review.user_id,
        date: review.date,
        rating: review.rating,
        title: review.title.clone(),
        description: review.description.clone(),
        venue: review.venue.clone(),
    };

    let Ok(updated_review) = Review::update(review_to_save) else {
        return HttpResponse::InternalServerError().json(Error {
            message: "Review could not be updated".to_string(),
        });
    };

    let imdb_id = if let Some(external_ids) = show.external_ids {
        external_ids.imdb_id
    } else {
        None
    };

    let show_review_to_save = ShowReview {
        review_id: existing_review.review_id,
        show_id: existing_review.show.id,
        user_id: auth.user_id,
        imdb_id,
        name: show.name,
        poster_path: show.poster_path,
        first_air_date: show.first_air_date,
    };

    let Ok(updated_show_review) = ShowReview::update(show_review_to_save) else {
        return HttpResponse::InternalServerError().json(Error {
            message: "Show review could not be updated".to_string(),
        });
    };

    let mut review_response = ShowReviewResponse {
        review_id: updated_review.review_id,
        user_id: updated_review.user_id,
        date: updated_review.date,
        rating: updated_review.rating,
        title: updated_review.title,
        description: updated_review.description,
        venue: updated_review.venue,
        show: Show::from(updated_show_review),
        company: None,
    };

    let Some(review_company) = review.company.clone() else {
        return HttpResponse::Ok().json(Success {
            data: review_response,
        });
    };

    let company =
        crate::review_company::ReviewCompany::replace(updated_review.review_id, review_company);

    let Ok(company) = company else {
        return HttpResponse::Ok().json(SuccessWithMessage {
            data: review_response,
            message: "Company could not be updated".to_string(),
        });
    };

    review_response.company = Some(company);

    HttpResponse::Ok().json(Success {
        data: review_response,
    })
}
