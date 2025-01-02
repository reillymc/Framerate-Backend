use crate::db::DbConnection;
use crate::schema::reviews;
use crate::user;
use crate::utils::AppError;
use chrono::{Datelike, Days, NaiveDate, Weekday};
use diesel::prelude::*;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(AsChangeset, Insertable, Associations, Selectable, Queryable)]
#[diesel(belongs_to(user::User))]
#[diesel(table_name = reviews)]
#[diesel(treat_none_as_null = true)]
pub struct Review {
    pub review_id: Uuid,
    pub user_id: Uuid,
    #[diesel(treat_none_as_null = true)]
    pub date: Option<NaiveDate>,
    pub rating: Option<i16>,
    pub title: Option<String>,
    pub description: Option<String>,
    pub venue: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum Order {
    Rating,
    Date,
    MediaTitle,
    MediaReleaseDate,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum Sort {
    Asc,
    Desc,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ReviewFindParameters {
    pub order_by: Option<Order>,
    pub sort: Option<Sort>,
    pub page: Option<i64>,
    pub page_size: Option<i64>,
    pub rating_min: Option<i16>,
    pub rating_max: Option<i16>,
    pub at_venue: Option<String>,
    pub with_company: Option<Uuid>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ReviewStatistics {
    pub reviews_this_week: i64,
    pub reviews_this_month: i64,
    pub reviews_this_year: i64,
}

impl Review {
    pub fn find(conn: &mut DbConnection, user_id: Uuid, review_id: Uuid) -> Result<Self, AppError> {
        let reviews = reviews::table
            .select(Review::as_select())
            .filter(reviews::review_id.eq(review_id))
            .filter(reviews::user_id.eq(user_id))
            .first(conn)?;
        Ok(reviews)
    }

    pub fn create(conn: &mut DbConnection, review: Review) -> Result<Self, AppError> {
        let new_review = diesel::insert_into(reviews::table)
            .values(review)
            .get_result(conn)?;
        Ok(new_review)
    }

    pub fn update(conn: &mut DbConnection, review: Review) -> Result<Self, AppError> {
        let updated_review = diesel::update(reviews::table)
            .filter(reviews::review_id.eq(review.review_id))
            .set(review)
            .get_result(conn)?;
        Ok(updated_review)
    }

    pub fn delete(conn: &mut DbConnection, review_id: Uuid) -> Result<usize, AppError> {
        let res = diesel::delete(reviews::table.filter(reviews::review_id.eq(review_id)))
            .execute(conn)?;
        Ok(res)
    }

    pub fn find_statistics(
        conn: &mut DbConnection,
        user_id: Uuid,
    ) -> Result<ReviewStatistics, AppError> {
        let current_year = chrono::offset::Local::now().year();
        let current_year_start = NaiveDate::from_ymd_opt(current_year, 1, 1).unwrap();
        let current_year_end = NaiveDate::from_ymd_opt(current_year + 1, 1, 1)
            .unwrap()
            .checked_sub_days(Days::new(1));

        let reviews_this_year: i64 = reviews::table
            .filter(reviews::user_id.eq(user_id))
            .filter(reviews::date.between(current_year_start, current_year_end))
            .count()
            .get_result(conn)?;

        let current_month = chrono::Utc::now().month();
        let current_month_start = NaiveDate::from_ymd_opt(current_year, current_month, 1).unwrap();
        let current_month_end = NaiveDate::from_ymd_opt(current_year, current_month + 1, 1)
            .unwrap()
            .checked_sub_days(Days::new(1));

        let reviews_this_month: i64 = reviews::table
            .filter(reviews::user_id.eq(user_id))
            .filter(reviews::date.between(current_month_start, current_month_end))
            .count()
            .get_result(conn)?;

        let current_week = chrono::Utc::now().iso_week().week();
        let current_week_start =
            NaiveDate::from_isoywd_opt(current_year, current_week, Weekday::Mon).unwrap();
        let current_week_end =
            NaiveDate::from_isoywd_opt(current_year, current_week + 1, Weekday::Mon).unwrap();

        let reviews_this_week: i64 = reviews::table
            .filter(reviews::user_id.eq(user_id))
            .filter(reviews::date.between(current_week_start, current_week_end))
            .count()
            .get_result(conn)?;

        Ok(ReviewStatistics {
            reviews_this_week,
            reviews_this_month,
            reviews_this_year,
        })
    }
}
