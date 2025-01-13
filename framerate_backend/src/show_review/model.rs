use chrono::NaiveDate;
use diesel::prelude::*;
use uuid::Uuid;

use crate::{
    db::{DbConnection, DEFAULT_PAGE_SIZE},
    review::{self, Review, ReviewFindParameters, ReviewOrder, ReviewSort},
    schema::{review_company, reviews, show_reviews},
    show::Show,
    user,
    utils::AppError,
};

#[derive(AsChangeset, Insertable, Associations, Selectable, Queryable)]
#[diesel(belongs_to(user::User))]
#[diesel(belongs_to(review::Review))]
#[diesel(table_name = show_reviews)]
pub struct ShowReview {
    pub review_id: Uuid,
    pub user_id: Uuid,
    pub show_id: i32,
    pub name: String,
    pub imdb_id: Option<String>,
    pub poster_path: Option<String>,
    #[diesel(treat_none_as_null = true)]
    pub first_air_date: Option<NaiveDate>,
}

pub struct ShowReviewReadResponse {
    pub review_id: Uuid,
    pub user_id: Uuid,
    pub date: Option<NaiveDate>,
    pub rating: Option<i16>,
    pub title: Option<String>,
    pub description: Option<String>,
    pub venue: Option<String>,
    pub show: Show,
}

impl From<ShowReview> for Show {
    fn from(show_review: ShowReview) -> Self {
        Show {
            id: show_review.show_id,
            name: show_review.name,
            poster_path: show_review.poster_path,
            first_air_date: show_review.first_air_date,
            last_air_date: None,
            next_air_date: None,
            status: None,
            backdrop_path: None,
            overview: None,
            popularity: None,
            external_ids: None,
            seasons: None,
            tagline: None,
            credits: None,
        }
    }
}

impl ShowReview {
    pub fn find_by_review_id(
        conn: &mut DbConnection,
        user_id: Uuid,
        review_id: Uuid,
    ) -> Result<ShowReviewReadResponse, AppError> {
        let (show_review, review_details) = show_reviews::table
            .filter(show_reviews::review_id.eq(review_id))
            .filter(show_reviews::user_id.eq(user_id))
            .inner_join(reviews::table)
            .select((ShowReview::as_select(), Review::as_select()))
            .first::<(ShowReview, Review)>(conn)?;

        let review = ShowReviewReadResponse {
            review_id: show_review.review_id,
            user_id: show_review.user_id,
            date: review_details.date,
            rating: review_details.rating,
            title: review_details.title,
            description: review_details.description,
            venue: review_details.venue,
            show: Show::from(show_review),
        };

        Ok(review)
    }

    pub fn find_all_reviews(
        conn: &mut DbConnection,
        user_id: Uuid,
        params: ReviewFindParameters,
    ) -> Result<Vec<ShowReviewReadResponse>, AppError> {
        let mut query = show_reviews::table
            .filter(show_reviews::user_id.eq(user_id))
            .inner_join(reviews::table)
            .filter(reviews::rating.is_not_null())
            .into_boxed();

        let order_by = params.order_by.unwrap_or(ReviewOrder::Date);
        let sort = params.sort.unwrap_or(ReviewSort::Desc);
        query = match sort {
            ReviewSort::Asc => match order_by {
                ReviewOrder::Date => query.order(reviews::date.asc().nulls_first()),
                ReviewOrder::MediaReleaseDate => {
                    query.order(show_reviews::first_air_date.asc().nulls_first())
                }
                ReviewOrder::Rating => query.order(reviews::rating.asc()),
                ReviewOrder::MediaTitle => query.order(show_reviews::name.asc().nulls_first()),
            },
            ReviewSort::Desc => match order_by {
                ReviewOrder::Date => query.order(reviews::date.desc().nulls_last()),
                ReviewOrder::MediaReleaseDate => {
                    query.order(show_reviews::first_air_date.desc().nulls_last())
                }
                ReviewOrder::Rating => query.order(reviews::rating.desc()),
                ReviewOrder::MediaTitle => query.order(show_reviews::name.desc().nulls_last()),
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
                        .filter(review_company::user_id.eq(with_company)),
                ),
            );
        }

        if let Some(page) = params.page {
            let page_size = params.page_size.unwrap_or(DEFAULT_PAGE_SIZE);
            query = query.limit(page_size).offset((page - 1) * page_size);
        }

        let reviews = query
            .select((ShowReview::as_select(), Review::as_select()))
            .load::<(ShowReview, Review)>(conn)?;

        let show_reviews: Vec<ShowReviewReadResponse> = reviews
            .into_iter()
            .map(|(show, review)| ShowReviewReadResponse {
                review_id: show.review_id,
                user_id: show.user_id,
                date: review.date,
                description: review.description,
                rating: review.rating,
                title: review.title,
                venue: review.venue,
                show: Show::from(show),
            })
            .collect();
        Ok(show_reviews)
    }

    pub fn find_by_show_id(
        conn: &mut DbConnection,
        user_id: Uuid,
        show_id: i32,
    ) -> Result<Vec<ShowReviewReadResponse>, AppError> {
        let reviews = show_reviews::table
            .filter(show_reviews::show_id.eq(show_id))
            .filter(show_reviews::user_id.eq(user_id))
            .inner_join(reviews::table)
            .order(reviews::date.desc().nulls_last())
            .select((ShowReview::as_select(), Review::as_select()))
            .load::<(ShowReview, Review)>(conn)?;

        let show_reviews: Vec<ShowReviewReadResponse> = reviews
            .into_iter()
            .map(|(show, review)| ShowReviewReadResponse {
                review_id: show.review_id,
                user_id: show.user_id,
                date: review.date,
                description: review.description,
                rating: review.rating,
                title: review.title,
                venue: review.venue,
                show: Show::from(show),
            })
            .collect();
        Ok(show_reviews)
    }

    pub fn create(conn: &mut DbConnection, review: ShowReview) -> Result<Self, AppError> {
        let new_review = diesel::insert_into(show_reviews::table)
            .values(review)
            .get_result(conn)?;
        Ok(new_review)
    }

    pub fn update(conn: &mut DbConnection, review: ShowReview) -> Result<Self, AppError> {
        let updated_review = diesel::update(show_reviews::table)
            .filter(show_reviews::review_id.eq(review.review_id))
            .set(review)
            .get_result(conn)?;
        Ok(updated_review)
    }
}
