use crate::error_handler::CustomError;
use crate::{db::establish_connection, schema::review_company};
use crate::{review, user};
use diesel::prelude::*;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Serialize, Deserialize, AsChangeset, Insertable, Associations, Selectable, Queryable)]
#[diesel(belongs_to(user::User))]
#[diesel(belongs_to(review::Review))]
#[diesel(table_name = review_company)]
#[serde(rename_all = "camelCase")]
pub struct ReviewCompany {
    pub review_id: Uuid,
    pub user_id: Uuid,
}

impl ReviewCompany {
    pub fn find_all(review_id: Uuid) -> Result<Self, CustomError> {
        let connection = &mut establish_connection();
        let review_company = review_company::table
            .filter(review_company::review_id.eq(review_id))
            .select(ReviewCompany::as_select())
            .first(connection)
            .expect("Error loading reviews");
        Ok(review_company)
    }

    pub fn create(review_id: Uuid, review_company: ReviewCompany) -> Result<Self, CustomError> {
        let review_company_to_save = ReviewCompany {
            review_id,
            user_id: review_company.user_id,
        };
        let connection = &mut establish_connection();
        let new_review = diesel::insert_into(review_company::table)
            .values(review_company_to_save)
            .get_result(connection)
            .expect("Error creating review");
        Ok(new_review)
    }

    pub fn delete(review_id: Uuid, user_id: Uuid) -> Result<usize, CustomError> {
        let connection = &mut establish_connection();
        let res = diesel::delete(
            review_company::table.filter(
                review_company::review_id
                    .eq(review_id)
                    .and(review_company::user_id.eq(user_id)),
            ),
        )
        .execute(connection)
        .expect("Error deleting review");
        Ok(res)
    }
}
