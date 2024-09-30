use crate::db::{establish_connection, DEFAULT_PAGE_SIZE};
use crate::error_handler::CustomError;
use crate::schema::{review_company, reviews};
use crate::user;
use chrono::{Datelike, Days, NaiveDate, Weekday};
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
    pub media_type: String,
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
    pub fn find(user_id: Uuid, review_id: Uuid) -> Result<Self, CustomError> {
        let connection = &mut establish_connection();
        let reviews = reviews::table
            .select(Review::as_select())
            .filter(reviews::review_id.eq(review_id))
            .filter(reviews::user_id.eq(user_id))
            .first(connection)?;
        Ok(reviews)
    }

    pub fn find_by_user(
        user_id: Uuid,
        params: ReviewFindParameters,
    ) -> Result<Vec<ReviewSummary>, CustomError> {
        let connection = &mut establish_connection();

        let mut query = reviews::table
            .filter(reviews::user_id.eq(user_id))
            .filter(reviews::media_type.eq(params.media_type))
            .into_boxed();

        let order_by = params.order_by.unwrap_or(Order::Date);
        let sort = params.sort.unwrap_or(Sort::Desc);
        query = match sort {
            Sort::Asc => match order_by {
                Order::Date => query.order(reviews::date.asc().nulls_first()),
                Order::MediaReleaseDate => {
                    query.order(reviews::media_release_date.asc().nulls_first())
                }
                Order::Rating => query.order(reviews::rating.asc()),
                Order::MediaTitle => query.order(reviews::media_title.asc().nulls_first()),
            },
            Sort::Desc => match order_by {
                Order::Date => query.order(reviews::date.desc().nulls_last()),
                Order::MediaReleaseDate => {
                    query.order(reviews::media_release_date.desc().nulls_last())
                }
                Order::Rating => query.order(reviews::rating.desc()),
                Order::MediaTitle => query.order(reviews::media_title.desc().nulls_last()),
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
            .load(connection)?;
        Ok(reviews)
    }

    pub fn find_by_media(user_id: Uuid, media_id: i32) -> Result<Vec<Self>, CustomError> {
        let connection = &mut establish_connection();
        let reviews = reviews::table
            .filter(reviews::media_id.eq(media_id))
            .filter(reviews::user_id.eq(user_id))
            .order(reviews::date.desc().nulls_last())
            .select(Review::as_select())
            .load(connection)?;
        Ok(reviews)
    }

    pub fn find_by_user_media(user_id: Uuid, media_id: i32) -> Result<Vec<Self>, CustomError> {
        let connection = &mut establish_connection();
        let reviews = reviews::table
            .filter(reviews::user_id.eq(user_id))
            .filter(reviews::media_id.eq(media_id))
            .order(reviews::date.desc().nulls_last())
            .select(Review::as_select())
            .load(connection)?;
        Ok(reviews)
    }

    pub fn create(review: Review) -> Result<Self, CustomError> {
        let connection = &mut establish_connection();
        let new_review = diesel::insert_into(reviews::table)
            .values(review)
            .get_result(connection)?;
        Ok(new_review)
    }

    pub fn update(review: Review) -> Result<Self, CustomError> {
        let connection = &mut establish_connection();
        let updated_review = diesel::update(reviews::table)
            .filter(reviews::review_id.eq(review.review_id))
            .set(review)
            .get_result(connection)?;
        Ok(updated_review)
    }

    pub fn delete(review_id: Uuid) -> Result<usize, CustomError> {
        let connection = &mut establish_connection();
        let res = diesel::delete(reviews::table.filter(reviews::review_id.eq(review_id)))
            .execute(connection)?;
        Ok(res)
    }

    pub fn find_statistics(user_id: Uuid) -> Result<ReviewStatistics, CustomError> {
        let connection = &mut establish_connection();

        let current_year = chrono::offset::Local::now().year();
        let current_year_start = NaiveDate::from_ymd_opt(current_year, 1, 1).unwrap();
        let current_year_end = NaiveDate::from_ymd_opt(current_year + 1, 1, 1)
            .unwrap()
            .checked_sub_days(Days::new(1));

        let reviews_this_year: i64 = reviews::table
            .filter(reviews::user_id.eq(user_id))
            .filter(reviews::date.between(current_year_start, current_year_end))
            .count()
            .get_result(connection)?;

        let current_month = chrono::Utc::now().month();
        let current_month_start = NaiveDate::from_ymd_opt(current_year, current_month, 1).unwrap();
        let current_month_end = NaiveDate::from_ymd_opt(current_year, current_month + 1, 1)
            .unwrap()
            .checked_sub_days(Days::new(1));

        let reviews_this_month: i64 = reviews::table
            .filter(reviews::user_id.eq(user_id))
            .filter(reviews::date.between(current_month_start, current_month_end))
            .count()
            .get_result(connection)?;

        let current_week = chrono::Utc::now().iso_week().week();
        let current_week_start =
            NaiveDate::from_isoywd_opt(current_year, current_week, Weekday::Mon).unwrap();
        let current_week_end =
            NaiveDate::from_isoywd_opt(current_year, current_week + 1, Weekday::Mon).unwrap();

        let reviews_this_week: i64 = reviews::table
            .filter(reviews::user_id.eq(user_id))
            .filter(reviews::date.between(current_week_start, current_week_end))
            .count()
            .get_result(connection)?;

        Ok(ReviewStatistics {
            reviews_this_week,
            reviews_this_month,
            reviews_this_year,
        })
    }
}
