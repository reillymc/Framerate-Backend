use super::MovieReviewReadResponse;

use crate::db::DbPool;
use crate::error_handler::CustomError;
use crate::movie::Movie;
use crate::movie_review::MovieReview;
use crate::review::{Review, ReviewFindParameters};
use crate::review_company::{ReviewCompany, ReviewCompanyDetails, ReviewCompanySummary};
use crate::utils::jwt::Auth;
use crate::utils::response_body::Success;
use actix_web::{get, post, web};
use actix_web::{put, Responder};
use chrono::NaiveDate;
use diesel::Connection;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SaveMovieReviewRequest {
    pub date: Option<NaiveDate>,
    pub rating: Option<i16>,
    pub title: Option<String>,
    pub description: Option<String>,
    pub venue: Option<String>,
    pub company: Option<Vec<ReviewCompanySummary>>,
}

impl SaveMovieReviewRequest {
    pub fn company(mut self, company: Vec<ReviewCompanySummary>) -> Self {
        self.company = Some(company);
        self
    }
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MovieReviewResponse {
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
    pub movie: Movie,
}

impl From<MovieReviewReadResponse> for MovieReviewResponse {
    fn from(review: MovieReviewReadResponse) -> Self {
        MovieReviewResponse {
            review_id: review.review_id,
            user_id: review.user_id,
            date: review.date,
            rating: review.rating,
            title: review.title,
            description: review.description,
            venue: review.venue,
            company: None,
            movie: review.movie,
        }
    }
}

impl MovieReviewResponse {
    pub fn company(mut self, company: Vec<ReviewCompanyDetails>) -> Self {
        self.company = Some(company);
        self
    }
}

#[get("/movies/reviews")]
async fn find_all(
    pool: web::Data<DbPool>,
    auth: Auth,
    params: web::Query<ReviewFindParameters>,
) -> actix_web::Result<impl Responder> {
    let reviews = web::block(move || {
        let mut conn = pool.get()?;
        MovieReview::find_all_reviews(&mut conn, auth.user_id, params.into_inner())
    })
    .await??;

    Ok(Success::new(
        reviews
            .into_iter()
            .map(MovieReviewResponse::from)
            .collect::<Vec<MovieReviewResponse>>(),
    ))
}

#[get("/movies/reviews/{review_id}")]
async fn find_by_review_id(
    pool: web::Data<DbPool>,
    auth: Auth,
    review_id: web::Path<Uuid>,
) -> actix_web::Result<impl Responder> {
    let review = web::block(move || {
        let mut conn = pool.get()?;
        let review =
            MovieReview::find_by_review_id(&mut conn, auth.user_id, review_id.into_inner())?;
        let company = ReviewCompany::find_by_review(&mut conn, review.review_id)?;

        Ok::<MovieReviewResponse, CustomError>(MovieReviewResponse::from(review).company(company))
    })
    .await??;

    Ok(Success::new(review))
}

#[get("/movies/{movie_id}/reviews")]
async fn find_by_movie_id(
    pool: web::Data<DbPool>,
    auth: Auth,
    movie_id: web::Path<i32>,
) -> actix_web::Result<impl Responder> {
    let reviews = web::block(move || {
        let mut conn = pool.get()?;
        MovieReview::find_by_movie_id(&mut conn, auth.user_id, movie_id.into_inner())
    })
    .await??;

    Ok(Success::new(
        reviews
            .into_iter()
            .map(MovieReviewResponse::from)
            .collect::<Vec<MovieReviewResponse>>(),
    ))
}

#[post("/movies/{movie_id}/reviews")]
async fn create(
    pool: web::Data<DbPool>,
    auth: Auth,
    review: web::Json<SaveMovieReviewRequest>,
    movie_id: web::Path<i32>,
) -> actix_web::Result<impl Responder> {
    let review = review.into_inner();
    let movie_id = movie_id.into_inner();

    let movie = Movie::find(&movie_id).await?;

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

    let movie_review_to_save = MovieReview {
        review_id,
        movie_id,
        user_id: auth.user_id,
        imdb_id: movie.imdb_id,
        title: movie.title,
        poster_path: movie.poster_path,
        release_date: movie.release_date,
    };

    let review = web::block(move || {
        let mut conn = pool.get()?;

        conn.transaction::<MovieReviewResponse, CustomError, _>(|mut conn| {
            let created_review = Review::create(&mut conn, review_to_save)?;
            let created_movie_review = MovieReview::create(&mut conn, movie_review_to_save)?;

            let company =
                ReviewCompany::replace(&mut conn, created_review.review_id, review.company)?;

            let review_response = MovieReviewResponse {
                review_id: created_review.review_id,
                user_id: created_review.user_id,
                date: created_review.date,
                rating: created_review.rating,
                title: created_review.title,
                description: created_review.description,
                venue: created_review.venue,
                movie: Movie::from(created_movie_review),
                company: Some(company),
            };

            Ok(review_response)
        })
    })
    .await??;

    Ok(Success::new(review))
}

#[put("/movies/{movie_id}/reviews/{review_id}")]
async fn update(
    pool: web::Data<DbPool>,
    auth: Auth,
    review: web::Json<SaveMovieReviewRequest>,
    path: web::Path<(i32, Uuid)>,
) -> actix_web::Result<impl Responder> {
    let (movie_id, review_id) = path.into_inner();

    let movie = Movie::find(&movie_id).await?;

    let review = web::block(move || {
        let mut conn = pool.get()?;

        let existing_review = MovieReview::find_by_review_id(&mut conn, auth.user_id, review_id)?;

        if movie_id != existing_review.movie.id {
            return Err(CustomError::new(400, "Review movie cannot be changed"));
        }

        let review_to_save = Review {
            review_id: existing_review.review_id,
            user_id: existing_review.user_id,
            date: review.date,
            rating: review.rating,
            title: review.title.clone(),
            description: review.description.clone(),
            venue: review.venue.clone(),
        };

        let movie_review_to_save = MovieReview {
            review_id: existing_review.review_id,
            movie_id: existing_review.movie.id,
            user_id: auth.user_id,
            imdb_id: movie.imdb_id,
            title: movie.title,
            poster_path: movie.poster_path,
            release_date: movie.release_date,
        };

        conn.transaction::<MovieReviewResponse, CustomError, _>(|mut conn| {
            let updated_review = Review::update(&mut conn, review_to_save)?;

            let updated_movie_review = MovieReview::update(&mut conn, movie_review_to_save)?;

            let company = ReviewCompany::replace(
                &mut conn,
                updated_review.review_id,
                review.company.clone(),
            )?;

            let review_response = MovieReviewResponse {
                review_id: updated_review.review_id,
                user_id: updated_review.user_id,
                date: updated_review.date,
                rating: updated_review.rating,
                title: updated_review.title,
                description: updated_review.description,
                venue: updated_review.venue,
                movie: Movie::from(updated_movie_review),
                company: Some(company),
            };

            Ok(review_response)
        })
    })
    .await??;

    Ok(Success::new(review))
}
