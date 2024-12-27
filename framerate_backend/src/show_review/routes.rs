use super::ShowReviewReadResponse;

use crate::db::DbPool;
use crate::review::{Review, ReviewFindParameters};
use crate::review_company::{ReviewCompany, ReviewCompanyDetails, ReviewCompanySummary};
use crate::show::Show;
use crate::show_review::ShowReview;
use crate::tmdb::TmdbClient;
use crate::utils::{jwt::Auth, response_body::Success, AppError};
use actix_web::{get, post, web};
use actix_web::{put, Responder};
use chrono::NaiveDate;
use diesel::Connection;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SaveShowReviewRequest {
    pub rating: Option<i16>,
    pub date: Option<NaiveDate>,
    pub title: Option<String>,
    pub description: Option<String>,
    pub venue: Option<String>,
    pub company: Option<Vec<ReviewCompanySummary>>,
}

impl SaveShowReviewRequest {
    pub fn company(mut self, company: Vec<ReviewCompanySummary>) -> Self {
        self.company = Some(company);
        self
    }
}

#[derive(Serialize, Deserialize)]
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

impl ShowReviewResponse {
    pub fn company(mut self, company: Vec<ReviewCompanyDetails>) -> Self {
        self.company = Some(company);
        self
    }
}

#[get("/shows/reviews")]
async fn find_all(
    pool: web::Data<DbPool>,
    auth: Auth,
    params: web::Query<ReviewFindParameters>,
) -> actix_web::Result<impl Responder> {
    let reviews = web::block(move || {
        let mut conn = pool.get()?;
        ShowReview::find_all_reviews(&mut conn, auth.user_id, params.into_inner())
    })
    .await??;

    Ok(Success::new(
        reviews
            .into_iter()
            .map(ShowReviewResponse::from)
            .collect::<Vec<ShowReviewResponse>>(),
    ))
}

#[get("/shows/reviews/{review_id}")]
async fn find_by_review_id(
    pool: web::Data<DbPool>,
    auth: Auth,
    review_id: web::Path<Uuid>,
) -> actix_web::Result<impl Responder> {
    let review = web::block(move || {
        let mut conn = pool.get()?;
        let review =
            ShowReview::find_by_review_id(&mut conn, auth.user_id, review_id.into_inner())?;
        let company = ReviewCompany::find_by_review(&mut conn, review.review_id)?;

        Ok::<ShowReviewResponse, AppError>(ShowReviewResponse::from(review).company(company))
    })
    .await??;

    Ok(Success::new(review))
}

#[get("/shows/{show_id}/reviews")]
async fn find_by_show_id(
    pool: web::Data<DbPool>,
    auth: Auth,
    show_id: web::Path<i32>,
) -> actix_web::Result<impl Responder> {
    let reviews = web::block(move || {
        let mut conn = pool.get()?;
        ShowReview::find_by_show_id(&mut conn, auth.user_id, show_id.into_inner())
    })
    .await??;

    Ok(Success::new(
        reviews
            .into_iter()
            .map(ShowReviewResponse::from)
            .collect::<Vec<ShowReviewResponse>>(),
    ))
}

#[post("/shows/{show_id}/reviews")]
async fn create(
    pool: web::Data<DbPool>,
    client: web::Data<TmdbClient>,
    auth: Auth,
    review: web::Json<SaveShowReviewRequest>,
    show_id: web::Path<i32>,
) -> actix_web::Result<impl Responder> {
    let show_id = show_id.into_inner();
    let review = review.into_inner();

    let show = Show::find(&client, &show_id).await?;

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

    let review = web::block(move || {
        let mut conn = pool.get()?;

        conn.transaction::<ShowReviewResponse, AppError, _>(|conn| {
            let created_review = Review::create(conn, review_to_save)?;
            let created_show_review = ShowReview::create(conn, show_review_to_save)?;

            let company =
                ReviewCompany::replace(conn, created_review.review_id, review.company.as_ref())?;

            let review_response = ShowReviewResponse {
                review_id: created_review.review_id,
                user_id: created_review.user_id,
                date: created_review.date,
                rating: created_review.rating,
                title: created_review.title,
                description: created_review.description,
                venue: created_review.venue,
                show: Show::from(created_show_review),
                company: Some(company),
            };

            Ok(review_response)
        })
    })
    .await??;

    Ok(Success::new(review))
}

#[put("/shows/{show_id}/reviews/{review_id}")]
async fn update(
    pool: web::Data<DbPool>,
    client: web::Data<TmdbClient>,
    auth: Auth,
    review: web::Json<SaveShowReviewRequest>,
    path: web::Path<(i32, Uuid)>,
) -> actix_web::Result<impl Responder> {
    let (show_id, review_id) = path.into_inner();
    let review = review.into_inner();

    let show = Show::find(&client, &show_id).await?;

    let review = web::block(move || {
        let mut conn = pool.get()?;

        let existing_review = ShowReview::find_by_review_id(&mut conn, auth.user_id, review_id)?;

        if show_id != existing_review.show.id {
            return Err(AppError::external(400, "Review show cannot be changed"));
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

        conn.transaction(|conn| {
            let updated_review = Review::update(conn, review_to_save)?;

            let updated_show_review = ShowReview::update(conn, show_review_to_save)?;

            let company =
                ReviewCompany::replace(conn, updated_review.review_id, review.company.as_ref())?;

            let review_response = ShowReviewResponse {
                review_id: updated_review.review_id,
                user_id: updated_review.user_id,
                date: updated_review.date,
                rating: updated_review.rating,
                title: updated_review.title,
                description: updated_review.description,
                venue: updated_review.venue,
                show: Show::from(updated_show_review),
                company: Some(company),
            };

            Ok(review_response)
        })
    })
    .await??;

    Ok(Success::new(review))
}
