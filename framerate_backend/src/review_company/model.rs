use crate::db::DbConnection;
use crate::schema::review_company;
use crate::schema::users;
use crate::user::User;
use crate::utils::AppError;
use crate::{review, user};
use diesel::prelude::*;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
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

#[derive(Serialize, Deserialize, AsChangeset, Associations, Selectable, Queryable, ToSchema)]
#[diesel(belongs_to(user::User))]
#[diesel(table_name = review_company)]
#[serde(rename_all = "camelCase")]
pub struct ReviewCompanySummary {
    pub user_id: Uuid,
}

impl From<ReviewCompany> for ReviewCompanySummary {
    fn from(review_company_summary: ReviewCompany) -> Self {
        ReviewCompanySummary {
            user_id: review_company_summary.user_id,
        }
    }
}

#[derive(Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct ReviewCompanyDetails {
    pub user_id: Uuid,
    pub first_name: String,
    pub last_name: String,
}

impl ReviewCompany {
    pub fn find_by_review(
        conn: &mut DbConnection,
        review_id: Uuid,
    ) -> Result<Vec<ReviewCompanyDetails>, AppError> {
        let review_company = review_company::table
            .filter(review_company::review_id.eq(review_id))
            .inner_join(users::table)
            .select((ReviewCompanySummary::as_select(), User::as_select()))
            .load::<(ReviewCompanySummary, User)>(conn)?;

        let review_company_details: Vec<ReviewCompanyDetails> = review_company
            .into_iter()
            .map(|(review_company_summary, user)| ReviewCompanyDetails {
                user_id: review_company_summary.user_id,
                first_name: user.first_name,
                last_name: user.last_name,
            })
            .collect();
        Ok(review_company_details)
    }

    pub fn replace(
        conn: &mut DbConnection,
        review_id: Uuid,
        review_company: Option<&Vec<ReviewCompanySummary>>,
    ) -> Result<Vec<ReviewCompanyDetails>, AppError> {
        conn.transaction::<_, diesel::result::Error, _>(|conn| {
            diesel::delete(review_company::table.filter(review_company::review_id.eq(review_id)))
                .execute(conn)?;

            let Some(review_company) = review_company else {
                return Ok(());
            };

            let review_company_items: Vec<ReviewCompany> = review_company
                .iter()
                .map(|review_company_summary| ReviewCompany {
                    review_id,
                    user_id: review_company_summary.user_id,
                })
                .collect();

            diesel::insert_into(review_company::table)
                .values(review_company_items)
                .execute(conn)?;
            Ok(())
        })?;

        let review_company_details = Self::find_by_review(conn, review_id)?;
        Ok(review_company_details)
    }
}
