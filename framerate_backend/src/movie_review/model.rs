use chrono::NaiveDate;
use diesel::prelude::*;
use uuid::Uuid;

use crate::{
    db::{DbConnection, DEFAULT_PAGE_SIZE},
    movie::Movie,
    review::{self, Review, ReviewFindParameters, ReviewOrder, ReviewSort},
    schema::{movie_reviews, review_company, reviews},
    user,
    utils::AppError,
};

#[derive(AsChangeset, Insertable, Associations, Selectable, Queryable)]
#[diesel(belongs_to(user::User))]
#[diesel(belongs_to(review::Review))]
#[diesel(table_name = movie_reviews)]
pub struct MovieReview {
    pub review_id: Uuid,
    pub user_id: Uuid,
    pub movie_id: i32,
    pub title: String,
    pub imdb_id: Option<String>,
    pub poster_path: Option<String>,
    #[diesel(treat_none_as_null = true)]
    pub release_date: Option<NaiveDate>,
}

pub struct MovieReviewReadResponse {
    pub review_id: Uuid,
    pub user_id: Uuid,
    pub date: Option<NaiveDate>,
    pub rating: Option<i16>,
    pub title: Option<String>,
    pub description: Option<String>,
    pub venue: Option<String>,
    pub movie: Movie,
}

impl From<MovieReview> for Movie {
    fn from(movie_review: MovieReview) -> Self {
        Movie {
            id: movie_review.movie_id,
            imdb_id: movie_review.imdb_id,
            title: movie_review.title,
            poster_path: movie_review.poster_path,
            release_date: movie_review.release_date,
            status: None,
            backdrop_path: None,
            overview: None,
            popularity: None,
            runtime: None,
            tagline: None,
            credits: None,
        }
    }
}

impl MovieReview {
    pub fn find_by_review_id(
        conn: &mut DbConnection,
        user_id: Uuid,
        review_id: Uuid,
    ) -> Result<MovieReviewReadResponse, AppError> {
        let (movie_review, review_details) = movie_reviews::table
            .filter(movie_reviews::review_id.eq(review_id))
            .filter(movie_reviews::user_id.eq(user_id))
            .inner_join(reviews::table)
            .select((MovieReview::as_select(), Review::as_select()))
            .first::<(MovieReview, Review)>(conn)?;

        let review = MovieReviewReadResponse {
            review_id: movie_review.review_id,
            user_id: movie_review.user_id,
            date: review_details.date,
            rating: review_details.rating,
            title: review_details.title,
            description: review_details.description,
            venue: review_details.venue,
            movie: Movie::from(movie_review),
        };

        Ok(review)
    }

    pub fn find_all_reviews(
        conn: &mut DbConnection,
        user_id: Uuid,
        params: ReviewFindParameters,
    ) -> Result<Vec<MovieReviewReadResponse>, AppError> {
        let mut query = movie_reviews::table
            .filter(movie_reviews::user_id.eq(user_id))
            .inner_join(reviews::table)
            .filter(reviews::rating.is_not_null())
            .into_boxed();

        let order_by = params.order_by.unwrap_or(ReviewOrder::Date);
        let sort = params.sort.unwrap_or(ReviewSort::Desc);
        query = match sort {
            ReviewSort::Asc => match order_by {
                ReviewOrder::Date => query.order(reviews::date.asc().nulls_first()),
                ReviewOrder::MediaReleaseDate => {
                    query.order(movie_reviews::release_date.asc().nulls_first())
                }
                ReviewOrder::Rating => query.order(reviews::rating.asc()),
                ReviewOrder::MediaTitle => query.order(movie_reviews::title.asc().nulls_first()),
            },
            ReviewSort::Desc => match order_by {
                ReviewOrder::Date => query.order(reviews::date.desc().nulls_last()),
                ReviewOrder::MediaReleaseDate => {
                    query.order(movie_reviews::release_date.desc().nulls_last())
                }
                ReviewOrder::Rating => query.order(reviews::rating.desc()),
                ReviewOrder::MediaTitle => query.order(movie_reviews::title.desc().nulls_last()),
            },
        };

        query = query.then_order_by(reviews::review_id.asc());

        if let Some(venue) = params.at_venue {
            query = query.filter(reviews::venue.eq(venue));
        }

        if let Some(rating_min) = params.rating_min {
            query = query.filter(reviews::rating.ge(rating_min));
        }

        if let Some(rating_max) = params.rating_max {
            query = query.filter(reviews::rating.le(rating_max));
        }

        if let Some(with_company) = params.with_company {
            query = query.filter(
                reviews::review_id.eq_any(
                    review_company::table
                        .select(review_company::review_id)
                        .filter(review_company::company_id.eq(with_company)),
                ),
            );
        }

        if let Some(page) = params.page {
            let page_size = params.page_size.unwrap_or(DEFAULT_PAGE_SIZE);
            query = query.limit(page_size).offset((page - 1) * page_size);
        }

        let reviews = query
            .select((MovieReview::as_select(), Review::as_select()))
            .load::<(MovieReview, Review)>(conn)?;

        let movie_reviews: Vec<MovieReviewReadResponse> = reviews
            .into_iter()
            .map(|(movie, review)| MovieReviewReadResponse {
                review_id: movie.review_id,
                user_id: movie.user_id,
                date: review.date,
                description: review.description,
                rating: review.rating,
                title: review.title,
                venue: review.venue,
                movie: Movie::from(movie),
            })
            .collect();
        Ok(movie_reviews)
    }

    pub fn find_by_movie_id(
        conn: &mut DbConnection,
        user_id: Uuid,
        movie_id: i32,
    ) -> Result<Vec<MovieReviewReadResponse>, AppError> {
        let reviews = movie_reviews::table
            .filter(movie_reviews::movie_id.eq(movie_id))
            .filter(movie_reviews::user_id.eq(user_id))
            .inner_join(reviews::table)
            .order(reviews::date.desc().nulls_last())
            .select((MovieReview::as_select(), Review::as_select()))
            .load::<(MovieReview, Review)>(conn)?;

        let movie_reviews: Vec<MovieReviewReadResponse> = reviews
            .into_iter()
            .map(|(movie, review)| MovieReviewReadResponse {
                review_id: movie.review_id,
                user_id: movie.user_id,
                date: review.date,
                description: review.description,
                rating: review.rating,
                title: review.title,
                venue: review.venue,
                movie: Movie::from(movie),
            })
            .collect();
        Ok(movie_reviews)
    }

    pub fn create(conn: &mut DbConnection, review: MovieReview) -> Result<Self, AppError> {
        let new_review = diesel::insert_into(movie_reviews::table)
            .values(review)
            .get_result(conn)?;
        Ok(new_review)
    }

    pub fn update(conn: &mut DbConnection, review: MovieReview) -> Result<Self, AppError> {
        let updated_review = diesel::update(movie_reviews::table)
            .filter(movie_reviews::review_id.eq(review.review_id))
            .set(review)
            .get_result(conn)?;
        Ok(updated_review)
    }
}
