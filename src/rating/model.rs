use crate::db::establish_connection;
use crate::error_handler::CustomError;
use crate::schema::ratings;
use crate::user;
use chrono::NaiveDate;
use diesel::prelude::*;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Serialize, Deserialize, AsChangeset, Insertable, Associations, Selectable, Queryable)]
#[diesel(belongs_to(user::User))]
#[diesel(table_name = ratings)]
pub struct Rating {
    pub rating_id: Uuid,
    pub user_id: Uuid,
    pub movie_id: i32,
    pub movie_title: String,
    pub movie_poster_uri: String,
    pub movie_release_year: i16,
    pub date: NaiveDate,
    pub value: i16,
    pub review_title: Option<String>,
    pub review_description: Option<String>,
}

impl Rating {
    pub fn find_all() -> Result<Vec<Self>, CustomError> {
        let connection = &mut establish_connection();
        let ratings = ratings::table
            .select(Rating::as_select())
            .load(connection)
            .expect("Error loading posts");
        Ok(ratings)
    }

    pub fn find(rating_id: Uuid) -> Result<Self, CustomError> {
        let connection = &mut establish_connection();
        let ratings = ratings::table
            .select(Rating::as_select())
            .filter(ratings::rating_id.eq(rating_id))
            .first(connection)
            .expect("Error loading posts");
        Ok(ratings)
    }

    pub fn find_by_user(user_id: Uuid) -> Result<Vec<Self>, CustomError> {
        let connection = &mut establish_connection();
        let ratings = ratings::table
            .filter(ratings::user_id.eq(user_id))
            .order(ratings::date.desc())
            .select(Rating::as_select())
            .load(connection)
            .expect("Error loading ratings");
        Ok(ratings)
    }

    pub fn find_by_movie(movie_id: i32) -> Result<Vec<Self>, CustomError> {
        let connection = &mut establish_connection();
        let ratings = ratings::table
            .filter(ratings::movie_id.eq(movie_id))
            .order(ratings::date.desc())
            .select(Rating::as_select())
            .load(connection)
            .expect("Error loading ratings");
        Ok(ratings)
    }

    pub fn find_by_user_movie(user_id: Uuid, movie_id: i32) -> Result<Vec<Self>, CustomError> {
        let connection = &mut establish_connection();
        let ratings = ratings::table
            .filter(ratings::user_id.eq(user_id))
            .filter(ratings::movie_id.eq(movie_id))
            .order(ratings::date.desc())
            .select(Rating::as_select())
            .load(connection)
            .expect("Error loading ratings");
        Ok(ratings)
    }

    pub fn create(rating: Rating) -> Result<Self, CustomError> {
        let connection = &mut establish_connection();
        let new_rating = diesel::insert_into(ratings::table)
            .values(rating)
            .get_result(connection)
            .expect("Error creating rating");
        Ok(new_rating)
    }

    pub fn update(id: Uuid, rating: Rating) -> Result<Self, CustomError> {
        let connection = &mut establish_connection();
        let updated_rating = diesel::update(ratings::table)
            .filter(ratings::rating_id.eq(id))
            .set(rating)
            .get_result(connection)
            .expect("Error updating rating");
        Ok(updated_rating)
    }

    pub fn delete(rating_id: Uuid) -> Result<usize, CustomError> {
        let connection = &mut establish_connection();
        let res = diesel::delete(ratings::table.filter(ratings::rating_id.eq(rating_id)))
            .execute(connection)
            .expect("Error deleting rating");
        Ok(res)
    }
}
