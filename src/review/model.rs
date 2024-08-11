use crate::error_handler::CustomError;
use crate::schema::reviews;
use crate::user;
use crate::{db::establish_connection, user::placeholder_user};
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
    pub media_release_year: i16,
    pub date: Option<NaiveDate>,
    pub rating: i16,
    pub review_title: Option<String>,
    pub review_description: Option<String>,
    pub venue: Option<String>,
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
    pub media_release_year: i16,
    pub date: Option<NaiveDate>,
    pub rating: i16,
    pub review_title: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NewReview {
    pub review_id: Option<Uuid>,
    pub media_id: i32,
    pub imdb_id: Option<String>,
    pub media_type: String,
    pub media_title: String,
    pub media_poster_uri: Option<String>,
    pub media_release_year: i16,
    pub date: Option<NaiveDate>,
    pub rating: i16,
    pub review_title: Option<String>,
    pub review_description: Option<String>,
    pub venue: Option<String>,
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

    pub fn find_by_user(user_id: Uuid) -> Result<Vec<ReviewSummary>, CustomError> {
        let connection = &mut establish_connection();
        let reviews = reviews::table
            .filter(reviews::user_id.eq(user_id))
            .order(reviews::date.desc().nulls_last())
            .select((
                reviews::review_id,
                reviews::user_id,
                reviews::media_id,
                reviews::media_title,
                reviews::media_type,
                reviews::media_poster_uri,
                reviews::media_release_year,
                reviews::date,
                reviews::rating,
                reviews::review_title,
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
            user_id: placeholder_user(),
            media_id: review.media_id,
            imdb_id: review.imdb_id,
            media_type: review.media_type,
            media_title: review.media_title,
            media_poster_uri: review.media_poster_uri,
            media_release_year: review.media_release_year,
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

    pub fn update(id: Uuid, review: NewReview) -> Result<Self, CustomError> {
        let review_to_save = Review {
            review_id: id,
            user_id: placeholder_user(),
            media_id: review.media_id,
            imdb_id: review.imdb_id,
            media_type: review.media_type,
            media_title: review.media_title,
            media_poster_uri: review.media_poster_uri,
            media_release_year: review.media_release_year,
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
