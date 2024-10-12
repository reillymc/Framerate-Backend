use super::SeasonReviewReadResponse;

use crate::review::Review;
use crate::review_company::{ReviewCompanyDetails, ReviewCompanySummary};
use crate::season::Season;
use crate::season_review::SeasonReview;
use crate::utils::jwt::Auth;
use crate::utils::response_body::{Error, Success, SuccessWithMessage};
use actix_web::{get, post, web, HttpResponse};
use actix_web::{put, Responder};
use chrono::NaiveDate;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SaveSeasonReviewRequest {
    pub date: Option<NaiveDate>,
    pub rating: i16,
    pub title: Option<String>,
    pub description: Option<String>,
    pub venue: Option<String>,
    pub company: Option<Vec<ReviewCompanySummary>>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SeasonReviewResponse {
    pub review_id: Uuid,
    pub user_id: Uuid,
    pub date: Option<NaiveDate>,
    pub rating: i16,
    pub title: Option<String>,
    pub description: Option<String>,
    pub venue: Option<String>,
    pub company: Option<Vec<ReviewCompanyDetails>>,
    pub season: Season,
}

impl From<SeasonReviewReadResponse> for SeasonReviewResponse {
    fn from(review: SeasonReviewReadResponse) -> Self {
        SeasonReviewResponse {
            review_id: review.review_id,
            user_id: review.user_id,
            date: review.date,
            rating: review.rating,
            title: review.title,
            description: review.description,
            venue: review.venue,
            company: None,
            season: review.season,
        }
    }
}

#[get("/shows/{show_id}/seasons/{season_number}/reviews")]
async fn find_by_show_season(auth: Auth, path: web::Path<(i32, i32)>) -> impl Responder {
    let (show_id, season_number) = path.into_inner();
    match SeasonReview::find_by_show_season(auth.user_id, show_id, season_number) {
        Err(err) => HttpResponse::InternalServerError().json(Error {
            message: err.message,
        }),
        Ok(reviews) => HttpResponse::Ok().json(Success {
            data: reviews
                .into_iter()
                .map(SeasonReviewResponse::from)
                .collect::<Vec<SeasonReviewResponse>>(),
        }),
    }
}

#[get("/shows/seasons/reviews/{review_id}")]
async fn find_by_review_id(auth: Auth, review_id: web::Path<Uuid>) -> impl Responder {
    let Ok(review) = SeasonReview::find_by_review_id(auth.user_id, review_id.into_inner()) else {
        return HttpResponse::NotFound().json(Error {
            message: "Review not found".to_string(),
        });
    };

    let company = crate::review_company::ReviewCompany::find_by_review(review.review_id);

    let Ok(company) = company else {
        return HttpResponse::Ok().json(SuccessWithMessage {
            data: SeasonReviewResponse::from(review),
            message: "Company could not be retrieved".to_string(),
        });
    };

    let mut review = SeasonReviewResponse::from(review);
    review.company = Some(company);

    HttpResponse::Ok().json(Success { data: review })
}

#[post("/shows/{show_id}/seasons/{season_number}/reviews")]
async fn create(
    auth: Auth,
    review: web::Json<SaveSeasonReviewRequest>,
    path: web::Path<(i32, i32)>,
) -> impl Responder {
    let review = review.into_inner();
    let (show_id, season_number) = path.into_inner();

    let Ok(season) = crate::season::Season::find(&show_id, &season_number).await else {
        return HttpResponse::NotFound().json(Error {
            message: "Season not found".to_string(),
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

    let season_review_to_save = SeasonReview {
        review_id,
        user_id: auth.user_id,
        show_id,
        season_number,
        name: season.name,
        poster_path: season.poster_path,
        air_date: season.air_date,
    };

    let Ok(created_season_review) = SeasonReview::create(season_review_to_save) else {
        return HttpResponse::InternalServerError().json(Error {
            message: "Season review could not be created".to_string(),
        });
    };

    let mut review_response = SeasonReviewResponse {
        review_id: created_review.review_id,
        user_id: created_review.user_id,
        date: created_review.date,
        rating: created_review.rating,
        title: created_review.title,
        description: created_review.description,
        venue: created_review.venue,
        season: Season::from(created_season_review),
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

#[put("shows/{show_id}/seasons/{season_number}/reviews/{review_id}")]
async fn update(
    auth: Auth,
    review: web::Json<SaveSeasonReviewRequest>,
    path: web::Path<(i32, i32, Uuid)>,
) -> impl Responder {
    let (_, _, review_id) = path.into_inner();

    let Ok(existing_review) = SeasonReview::find_by_review_id(auth.user_id, review_id) else {
        return HttpResponse::NotFound().json(Error {
            message: "Review not found".to_string(),
        });
    };

    let Ok(season) = crate::season::Season::find(
        &existing_review.season.show_id,
        &existing_review.season.season_number,
    )
    .await
    else {
        return HttpResponse::NotFound().json(Error {
            message: "Season not found".to_string(),
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

    let season_review_to_save = SeasonReview {
        review_id: existing_review.review_id,
        user_id: auth.user_id,
        show_id: existing_review.season.show_id,
        season_number: existing_review.season.season_number,
        name: season.name,
        poster_path: season.poster_path,
        air_date: season.air_date,
    };

    let Ok(updated_season_review) = SeasonReview::update(season_review_to_save) else {
        return HttpResponse::InternalServerError().json(Error {
            message: "Season review could not be updated".to_string(),
        });
    };

    let mut review_response = SeasonReviewResponse {
        review_id: updated_review.review_id,
        user_id: updated_review.user_id,
        date: updated_review.date,
        rating: updated_review.rating,
        title: updated_review.title,
        description: updated_review.description,
        venue: updated_review.venue,
        season: Season::from(updated_season_review),
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
