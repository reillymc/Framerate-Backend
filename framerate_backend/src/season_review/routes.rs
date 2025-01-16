use super::SeasonReviewReadResponse;

use crate::db::DbPool;
use crate::review::Review;
use crate::review_company::{ReviewCompany, ReviewCompanyDetails, ReviewCompanySummary};
use crate::season::Season;
use crate::season_review::SeasonReview;
use crate::tmdb::TmdbClient;
use crate::utils::{jwt::Auth, response_body::Success, AppError};
use actix_web::{get, post, web};
use actix_web::{put, Responder};
use chrono::NaiveDate;
use diesel::Connection;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::Uuid;

#[derive(Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct SaveSeasonReviewRequest {
    #[schema(nullable = false)]
    pub date: Option<NaiveDate>,
    #[schema(nullable = false)]
    pub rating: Option<i16>,
    #[schema(nullable = false)]
    pub title: Option<String>,
    #[schema(nullable = false)]
    pub description: Option<String>,
    #[schema(nullable = false)]
    pub venue: Option<String>,
    #[schema(nullable = false)]
    pub company: Option<Vec<ReviewCompanySummary>>,
}

impl SaveSeasonReviewRequest {
    pub fn company(mut self, company: Vec<ReviewCompanySummary>) -> Self {
        self.company = Some(company);
        self
    }
}

#[derive(Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct SeasonReviewResponse {
    pub review_id: Uuid,
    pub user_id: Uuid,
    #[schema(nullable = false)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub date: Option<NaiveDate>,
    #[schema(nullable = false)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub rating: Option<i16>,
    #[schema(nullable = false)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,
    #[schema(nullable = false)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[schema(nullable = false)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub venue: Option<String>,
    #[schema(nullable = false)]
    #[serde(skip_serializing_if = "Option::is_none")]
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

impl SeasonReviewResponse {
    pub fn company(mut self, company: Vec<ReviewCompanyDetails>) -> Self {
        self.company = Some(company);
        self
    }
}

#[utoipa::path(tag = "Season Review", responses((status = OK, body = Vec<SeasonReviewResponse>)))]
#[get("/shows/{show_id}/seasons/{season_number}/reviews")]
async fn find_by_show_season(
    pool: web::Data<DbPool>,
    auth: Auth,
    path: web::Path<(i32, i32)>,
) -> actix_web::Result<impl Responder> {
    let (show_id, season_number) = path.into_inner();

    let reviews = web::block(move || {
        let mut conn = pool.get()?;
        SeasonReview::find_by_show_season(&mut conn, auth.user_id, show_id, season_number)
    })
    .await??;

    Ok(Success::new(
        reviews
            .into_iter()
            .map(SeasonReviewResponse::from)
            .collect::<Vec<SeasonReviewResponse>>(),
    ))
}

#[utoipa::path(tag = "Season Review", responses((status = OK, body = SeasonReviewResponse)))]
#[get("/shows/seasons/reviews/{review_id}")]
async fn find_by_review_id(
    pool: web::Data<DbPool>,
    auth: Auth,
    review_id: web::Path<Uuid>,
) -> actix_web::Result<impl Responder> {
    let review = web::block(move || {
        let mut conn = pool.get()?;
        let review =
            SeasonReview::find_by_review_id(&mut conn, auth.user_id, review_id.into_inner())?;
        let company = ReviewCompany::find_by_review(&mut conn, review.review_id)?;

        Ok::<SeasonReviewResponse, AppError>(SeasonReviewResponse::from(review).company(company))
    })
    .await??;

    Ok(Success::new(review))
}

#[utoipa::path(tag = "Season Review", responses((status = OK, body = SeasonReviewResponse)))]
#[post("/shows/{show_id}/seasons/{season_number}/reviews")]
async fn create(
    pool: web::Data<DbPool>,
    client: web::Data<TmdbClient>,
    auth: Auth,
    review: web::Json<SaveSeasonReviewRequest>,
    path: web::Path<(i32, i32)>,
) -> actix_web::Result<impl Responder> {
    let (show_id, season_number) = path.into_inner();
    let review = review.into_inner();

    let season = Season::find(&client, &show_id, &season_number).await?;

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

    let season_review_to_save = SeasonReview {
        review_id,
        user_id: auth.user_id,
        show_id,
        season_number,
        name: season.name,
        poster_path: season.poster_path,
        air_date: season.air_date,
    };

    let review = web::block(move || {
        let mut conn = pool.get()?;

        conn.transaction::<SeasonReviewResponse, AppError, _>(|conn| {
            let created_review = Review::create(conn, review_to_save)?;
            let created_season_review = SeasonReview::create(conn, season_review_to_save)?;

            let company =
                ReviewCompany::replace(conn, created_review.review_id, review.company.as_ref())?;

            let review_response = SeasonReviewResponse {
                review_id: created_review.review_id,
                user_id: created_review.user_id,
                date: created_review.date,
                rating: created_review.rating,
                title: created_review.title,
                description: created_review.description,
                venue: created_review.venue,
                season: Season::from(created_season_review),
                company: Some(company),
            };

            Ok(review_response)
        })
    })
    .await??;

    Ok(Success::new(review))
}

#[utoipa::path(tag = "Season Review", responses((status = OK, body = SeasonReviewResponse)))]
#[put("/shows/{show_id}/seasons/{season_number}/reviews/{review_id}")]
async fn update(
    pool: web::Data<DbPool>,
    client: web::Data<TmdbClient>,
    auth: Auth,
    review: web::Json<SaveSeasonReviewRequest>,
    path: web::Path<(i32, i32, Uuid)>,
) -> actix_web::Result<impl Responder> {
    let (show_id, season_number, review_id) = path.into_inner();
    let review = review.into_inner();

    let season = Season::find(&client, &show_id, &season_number).await?;

    let review = web::block(move || {
        let mut conn = pool.get()?;

        let existing_review = SeasonReview::find_by_review_id(&mut conn, auth.user_id, review_id)?;

        if show_id != existing_review.season.show_id
            || season_number != existing_review.season.season_number
        {
            return Err(AppError::external(
                400,
                "Review show and season cannot be changed",
            ));
        }

        let review_to_save = Review {
            review_id: existing_review.review_id,
            user_id: existing_review.user_id,
            date: review.date,
            rating: review.rating,
            title: review.title,
            description: review.description,
            venue: review.venue,
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

        conn.transaction(|conn| {
            let updated_review = Review::update(conn, review_to_save)?;

            let updated_season_review = SeasonReview::update(conn, season_review_to_save)?;

            let company =
                ReviewCompany::replace(conn, updated_review.review_id, review.company.as_ref())?;

            let review_response = SeasonReviewResponse {
                review_id: updated_review.review_id,
                user_id: updated_review.user_id,
                date: updated_review.date,
                rating: updated_review.rating,
                title: updated_review.title,
                description: updated_review.description,
                venue: updated_review.venue,
                season: Season::from(updated_season_review),
                company: Some(company),
            };

            Ok(review_response)
        })
    })
    .await??;

    Ok(Success::new(review))
}
