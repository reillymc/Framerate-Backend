use chrono::NaiveDate;
use diesel::prelude::*;
use uuid::Uuid;

use crate::{
    db::DbConnection,
    error_handler::CustomError,
    review::{self, Review},
    schema::{reviews, season_reviews},
    season::Season,
    user,
};

#[derive(AsChangeset, Insertable, Associations, Selectable, Queryable)]
#[diesel(belongs_to(user::User))]
#[diesel(belongs_to(review::Review))]
#[diesel(table_name = season_reviews)]
pub struct SeasonReview {
    pub review_id: Uuid,
    pub user_id: Uuid,
    pub show_id: i32,
    pub season_number: i32,
    pub name: Option<String>,
    pub poster_path: Option<String>,
    #[diesel(treat_none_as_null = true)]
    pub air_date: Option<NaiveDate>,
}

pub struct SeasonReviewReadResponse {
    pub review_id: Uuid,
    pub user_id: Uuid,
    pub date: Option<NaiveDate>,
    pub rating: Option<i16>,
    pub title: Option<String>,
    pub description: Option<String>,
    pub venue: Option<String>,
    pub season: Season,
}

impl From<SeasonReview> for Season {
    fn from(season_review: SeasonReview) -> Self {
        Season {
            show_id: season_review.show_id,
            season_number: season_review.season_number,
            name: season_review.name,
            poster_path: season_review.poster_path,
            air_date: season_review.air_date,
            overview: None,
            episode_count: None,
            episodes: None,
        }
    }
}

impl SeasonReview {
    pub fn find_by_review_id(
        conn: &mut DbConnection,
        user_id: Uuid,
        review_id: Uuid,
    ) -> Result<SeasonReviewReadResponse, CustomError> {
        let (season_review, review_details) = season_reviews::table
            .filter(season_reviews::review_id.eq(review_id))
            .filter(season_reviews::user_id.eq(user_id))
            .inner_join(reviews::table)
            .select((SeasonReview::as_select(), Review::as_select()))
            .first::<(SeasonReview, Review)>(conn)?;

        let review = SeasonReviewReadResponse {
            review_id: season_review.review_id,
            user_id: season_review.user_id,
            date: review_details.date,
            rating: review_details.rating,
            title: review_details.title,
            description: review_details.description,
            venue: review_details.venue,
            season: Season::from(season_review),
        };

        Ok(review)
    }

    pub fn find_by_show_season(
        conn: &mut DbConnection,
        user_id: Uuid,
        show_id: i32,
        season_number: i32,
    ) -> Result<Vec<SeasonReviewReadResponse>, CustomError> {
        let reviews = season_reviews::table
            .filter(season_reviews::show_id.eq(show_id))
            .filter(season_reviews::season_number.eq(season_number))
            .filter(season_reviews::user_id.eq(user_id))
            .inner_join(reviews::table)
            .order(reviews::date.desc().nulls_last())
            .select((SeasonReview::as_select(), Review::as_select()))
            .load::<(SeasonReview, Review)>(conn)?;

        let season_reviews: Vec<SeasonReviewReadResponse> = reviews
            .into_iter()
            .map(|(season, review)| SeasonReviewReadResponse {
                review_id: season.review_id,
                user_id: season.user_id,
                date: review.date,
                description: review.description,
                rating: review.rating,
                title: review.title,
                venue: review.venue,
                season: Season::from(season),
            })
            .collect();
        Ok(season_reviews)
    }

    pub fn create(conn: &mut DbConnection, review: SeasonReview) -> Result<Self, CustomError> {
        let new_review = diesel::insert_into(season_reviews::table)
            .values(review)
            .get_result(conn)?;
        Ok(new_review)
    }

    pub fn update(conn: &mut DbConnection, review: SeasonReview) -> Result<Self, CustomError> {
        let updated_review = diesel::update(season_reviews::table)
            .filter(season_reviews::review_id.eq(review.review_id))
            .set(review)
            .get_result(conn)?;
        Ok(updated_review)
    }
}
