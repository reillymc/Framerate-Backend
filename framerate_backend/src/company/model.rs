use crate::{db::DbConnection, schema::company, user, utils::AppError};
use chrono::NaiveDateTime;
use diesel::prelude::*;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::Uuid;

#[derive(Serialize, Deserialize, Selectable, Queryable, Associations, Insertable, ToSchema)]
#[diesel(belongs_to(user::User))]
#[diesel(table_name = company)]
#[serde(rename_all = "camelCase")]
pub struct Company {
    pub company_id: Uuid,
    pub first_name: String,
    pub last_name: String,
    pub date_created: NaiveDateTime,
    pub created_by: Uuid,
    #[schema(nullable = false)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub user_id: Option<Uuid>,
}

#[derive(AsChangeset, Insertable, Serialize, Deserialize, ToSchema)]
#[diesel(table_name = company)]
#[serde(rename_all = "camelCase")]
pub struct SaveCompany {
    pub first_name: String,
    pub last_name: String,
    #[schema(nullable = false)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub user_id: Option<Uuid>,
}

impl Company {
    pub fn find_all(conn: &mut DbConnection, created_by: &Uuid) -> Result<Vec<Self>, AppError> {
        let company = company::table
            .select(Company::as_select())
            .filter(company::created_by.eq(created_by))
            .load(conn)?;
        Ok(company)
    }

    pub fn create(
        conn: &mut DbConnection,
        company: SaveCompany,
        created_by: Uuid,
    ) -> Result<Self, AppError> {
        let company_to_save = Company {
            company_id: Uuid::new_v4(),
            first_name: company.first_name,
            last_name: company.last_name,
            date_created: chrono::Local::now().naive_local(),
            created_by,
            user_id: company.user_id,
        };

        let new_company: Company = diesel::insert_into(company::table)
            .values(company_to_save)
            .get_result(conn)?;

        Ok(new_company.into())
    }

    pub fn update(
        conn: &mut DbConnection,
        company_id: Uuid,
        company: SaveCompany,
        created_by: &Uuid,
    ) -> Result<Self, AppError> {
        let updated_company: Company = diesel::update(company::table)
            .filter(company::created_by.eq(created_by))
            .filter(company::company_id.eq(company_id))
            .set(company)
            .get_result(conn)?;
        Ok(updated_company.into())
    }

    pub fn delete(
        conn: &mut DbConnection,
        company_id: Uuid,
        created_by: &Uuid,
    ) -> Result<usize, AppError> {
        let res = diesel::delete(company::table)
            .filter(company::created_by.eq(created_by))
            .filter(company::company_id.eq(company_id))
            .execute(conn)?;
        Ok(res)
    }
}
