use super::MovieReviewReadResponse;

use crate::movie::Movie;
use crate::movie_review::MovieReview;
use crate::review::{Review, ReviewFindParameters};
use crate::review_company::{ReviewCompanyDetails, ReviewCompanySummary};
use crate::utils::jwt::Auth;
use crate::utils::response_body::{Error, Success, SuccessWithMessage};
use actix_web::{get, post, web, HttpResponse};
use actix_web::{put, Responder};
use chrono::NaiveDate;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SaveMovieReviewRequest {
    pub date: Option<NaiveDate>,
    pub rating: Option<i16>,
    pub title: Option<String>,
    pub description: Option<String>,
    pub venue: Option<String>,
    pub company: Option<Vec<ReviewCompanySummary>>,
}

#[derive(Serialize)]
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

#[get("/movies/reviews")]
async fn find_all(auth: Auth, params: web::Query<ReviewFindParameters>) -> impl Responder {
    match MovieReview::find_all_reviews(auth.user_id, params.into_inner()) {
        Err(err) => HttpResponse::InternalServerError().json(Error {
            message: err.message,
        }),
        Ok(reviews) => HttpResponse::Ok().json(Success {
            data: reviews
                .into_iter()
                .map(MovieReviewResponse::from)
                .collect::<Vec<MovieReviewResponse>>(),
        }),
    }
}

#[get("/movies/reviews/{review_id}")]
async fn find_by_review_id(auth: Auth, review_id: web::Path<Uuid>) -> impl Responder {
    let Ok(review) = MovieReview::find_by_review_id(auth.user_id, review_id.into_inner()) else {
        return HttpResponse::NotFound().json(Error {
            message: "Review not found".to_string(),
        });
    };

    let company = crate::review_company::ReviewCompany::find_by_review(review.review_id);

    let Ok(company) = company else {
        return HttpResponse::Ok().json(SuccessWithMessage {
            data: MovieReviewResponse::from(review),
            message: "Company could not be retrieved".to_string(),
        });
    };

    let mut review = MovieReviewResponse::from(review);
    review.company = Some(company);

    HttpResponse::Ok().json(Success { data: review })
}

#[get("/movies/{movie_id}/reviews")]
async fn find_by_movie_id(auth: Auth, movie_id: web::Path<i32>) -> impl Responder {
    match MovieReview::find_by_movie_id(auth.user_id, movie_id.into_inner()) {
        Err(err) => HttpResponse::InternalServerError().json(Error {
            message: err.message,
        }),
        Ok(reviews) => HttpResponse::Ok().json(Success {
            data: reviews
                .into_iter()
                .map(MovieReviewResponse::from)
                .collect::<Vec<MovieReviewResponse>>(),
        }),
    }
}

#[post("/movies/{movie_id}/reviews")]
async fn create(
    auth: Auth,
    review: web::Json<SaveMovieReviewRequest>,
    movie_id: web::Path<i32>,
) -> impl Responder {
    let review = review.into_inner();
    let movie_id = movie_id.into_inner();

    let Ok(movie) = crate::movie::Movie::find(&movie_id).await else {
        return HttpResponse::NotFound().json(Error {
            message: "Movie not found".to_string(),
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

    let movie_review_to_save = MovieReview {
        review_id,
        movie_id,
        user_id: auth.user_id,
        imdb_id: movie.imdb_id,
        title: movie.title,
        poster_path: movie.poster_path,
        release_date: movie.release_date,
    };

    let Ok(created_movie_review) = MovieReview::create(movie_review_to_save) else {
        return HttpResponse::InternalServerError().json(Error {
            message: "Movie review could not be created".to_string(),
        });
    };

    let mut review_response = MovieReviewResponse {
        review_id: created_review.review_id,
        user_id: created_review.user_id,
        date: created_review.date,
        rating: created_review.rating,
        title: created_review.title,
        description: created_review.description,
        venue: created_review.venue,
        movie: Movie::from(created_movie_review),
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

#[put("/movies/{movie_id}/reviews/{review_id}")]
async fn update(
    auth: Auth,
    review: web::Json<SaveMovieReviewRequest>,
    path: web::Path<(i32, Uuid)>,
) -> impl Responder {
    let (_, review_id) = path.into_inner();
    let Ok(existing_review) = MovieReview::find_by_review_id(auth.user_id, review_id) else {
        return HttpResponse::NotFound().json(Error {
            message: "Review not found".to_string(),
        });
    };

    let Ok(movie) = crate::movie::Movie::find(&existing_review.movie.id).await else {
        return HttpResponse::NotFound().json(Error {
            message: "Movie not found".to_string(),
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

    let movie_review_to_save = MovieReview {
        review_id: existing_review.review_id,
        movie_id: existing_review.movie.id,
        user_id: auth.user_id,
        imdb_id: movie.imdb_id,
        title: movie.title,
        poster_path: movie.poster_path,
        release_date: movie.release_date,
    };

    let Ok(updated_movie_review) = MovieReview::update(movie_review_to_save) else {
        return HttpResponse::InternalServerError().json(Error {
            message: "Movie review could not be updated".to_string(),
        });
    };

    let mut review_response = MovieReviewResponse {
        review_id: updated_review.review_id,
        user_id: updated_review.user_id,
        date: updated_review.date,
        rating: updated_review.rating,
        title: updated_review.title,
        description: updated_review.description,
        venue: updated_review.venue,
        movie: Movie::from(updated_movie_review),
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
