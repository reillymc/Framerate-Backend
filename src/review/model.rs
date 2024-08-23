use crate::db::{establish_connection, DEFAULT_PAGE_SIZE};
use crate::error_handler::CustomError;
use crate::schema::{review_company, reviews};
use crate::user;
use chrono::NaiveDate;
use diesel::prelude::*;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Serialize, Deserialize, AsChangeset, Insertable, Associations, Selectable, Queryable)]
#[diesel(belongs_to(user::User))]
#[diesel(table_name = reviews)]
#[serde(rename_all = "camelCase")]
pub struct Review {
    pub review_id: Uuid,
    pub user_id: Uuid,
    pub media_id: i32,
    pub imdb_id: Option<String>,
    pub media_type: String,
    pub media_title: String,
    pub media_poster_uri: Option<String>,
    #[diesel(treat_none_as_null = true)]
    pub date: Option<NaiveDate>,
    pub rating: i16,
    pub review_title: Option<String>,
    pub review_description: Option<String>,
    pub venue: Option<String>,
    #[diesel(treat_none_as_null = true)]
    pub media_release_date: Option<NaiveDate>,
}

#[derive(Queryable, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ReviewSummary {
    pub review_id: Uuid,
    pub user_id: Uuid,
    pub media_id: i32,
    pub media_title: String,
    pub media_type: String,
    pub media_poster_uri: Option<String>,
    pub date: Option<NaiveDate>,
    pub rating: i16,
    pub review_description: Option<String>,
    pub venue: Option<String>,
    pub media_release_date: Option<NaiveDate>,
}

#[derive(Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NewReview {
    pub review_id: Option<Uuid>,
    pub media_id: i32,
    pub imdb_id: Option<String>,
    pub user_id: Uuid,
    pub media_type: String,
    pub media_title: String,
    pub media_poster_uri: Option<String>,
    pub date: Option<NaiveDate>,
    pub rating: i16,
    pub review_title: Option<String>,
    pub review_description: Option<String>,
    pub venue: Option<String>,
    pub media_release_date: Option<NaiveDate>,
}

#[derive(Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdatedReview {
    pub review_id: Uuid,
    pub media_id: i32,
    pub imdb_id: Option<String>,
    pub user_id: Uuid,
    pub media_type: String,
    pub media_title: String,
    pub media_poster_uri: Option<String>,
    pub media_release_date: Option<NaiveDate>,
    pub date: Option<NaiveDate>,
    pub rating: i16,
    pub review_title: Option<String>,
    pub review_description: Option<String>,
    pub venue: Option<String>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ReviewFindParameters {
    pub order_by: Option<String>,
    pub sort: Option<String>,
    pub page: Option<i64>,
    pub page_size: Option<i64>,
    pub rating_min: Option<i16>,
    pub rating_max: Option<i16>,
    pub at_venue: Option<String>,
    pub with_company: Option<Uuid>,
}

impl Review {
    pub fn find(review_id: Uuid) -> Result<Self, CustomError> {
        let connection = &mut establish_connection();
        let reviews = reviews::table
            .select(Review::as_select())
            .filter(reviews::review_id.eq(review_id))
            .first(connection)
            .expect("Error loading reviews");
        Ok(reviews)
    }

    pub fn find_by_user(
        user_id: Uuid,
        params: ReviewFindParameters,
    ) -> Result<Vec<ReviewSummary>, CustomError> {
        let connection = &mut establish_connection();

        let mut query = reviews::table
            .filter(reviews::user_id.eq(user_id))
            .into_boxed();

        if let Some(order_by) = params.order_by {
            let order = params.sort.unwrap_or_else(|| "asc".to_string());
            query = match order.as_str() {
                "asc" => match order_by.as_str() {
                    "date" => query.order(reviews::date.asc().nulls_last()),
                    "rating" => query.order(reviews::rating.asc()),
                    "title" => query.order(reviews::media_title.asc()),
                    _ => query,
                },
                "desc" => match order_by.as_str() {
                    "date" => query.order(reviews::date.desc().nulls_last()),
                    "rating" => query.order(reviews::rating.desc()),
                    "title" => query.order(reviews::media_title.desc()),
                    _ => query,
                },
                _ => query,
            };
        }

        if let Some(venue) = params.at_venue {
            println!("Venue: {}", venue);
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
            .select((
                reviews::review_id,
                reviews::user_id,
                reviews::media_id,
                reviews::media_title,
                reviews::media_type,
                reviews::media_poster_uri,
                reviews::date,
                reviews::rating,
                reviews::review_description,
                reviews::venue,
                reviews::media_release_date,
            ))
            .load(connection)
            .expect("Error loading reviews");
        Ok(reviews)
    }

    pub fn find_by_media(media_id: i32) -> Result<Vec<Self>, CustomError> {
        let connection = &mut establish_connection();
        let reviews = reviews::table
            .filter(reviews::media_id.eq(media_id))
            .order(reviews::date.desc().nulls_last())
            .select(Review::as_select())
            .load(connection)
            .expect("Error loading reviews");
        Ok(reviews)
    }

    pub fn find_by_user_media(user_id: Uuid, media_id: i32) -> Result<Vec<Self>, CustomError> {
        let connection = &mut establish_connection();
        let reviews = reviews::table
            .filter(reviews::user_id.eq(user_id))
            .filter(reviews::media_id.eq(media_id))
            .order(reviews::date.desc().nulls_last())
            .select(Review::as_select())
            .load(connection)
            .expect("Error loading reviews");
        Ok(reviews)
    }

    pub fn create(review: NewReview) -> Result<Self, CustomError> {
        let review_to_save = Review {
            review_id: Uuid::new_v4(),
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
        };

        let connection = &mut establish_connection();
        let new_review = diesel::insert_into(reviews::table)
            .values(review_to_save)
            .get_result(connection)
            .expect("Error creating review");
        Ok(new_review)
    }

    pub fn update(id: Uuid, review: UpdatedReview) -> Result<Self, CustomError> {
        let review_to_save = Review {
            review_id: id,
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
        };

        let connection = &mut establish_connection();
        let updated_review = diesel::update(reviews::table)
            .filter(reviews::review_id.eq(id))
            .set(review_to_save)
            .get_result(connection)
            .expect("Error updating review");
        Ok(updated_review)
    }

    pub fn delete(review_id: Uuid) -> Result<usize, CustomError> {
        let connection = &mut establish_connection();
        let res = diesel::delete(reviews::table.filter(reviews::review_id.eq(review_id)))
            .execute(connection)
            .expect("Error deleting review");
        Ok(res)
    }
}
