use crate::db::DbConnection;
use crate::schema::collections;
use crate::user;
use crate::utils::AppError;
use diesel::prelude::*;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::Uuid;

#[derive(AsChangeset, Insertable, Associations, Selectable, Queryable)]
#[diesel(belongs_to(user::User))]
#[diesel(table_name = collections)]
pub struct Collection {
    pub collection_id: Uuid,
    pub user_id: Uuid,
    pub name: String,
    pub media_type: String,
    pub default_for: Option<String>,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct NewCollection {
    pub name: String,
    pub media_type: String,
}

#[derive(Debug, Deserialize, Serialize, AsChangeset, ToSchema)]
#[diesel(table_name = collections)]
#[serde(rename_all = "camelCase")]
pub struct UpdatedCollection {
    pub name: String,
}

impl Collection {
    pub fn find_default(
        conn: &mut DbConnection,
        user_id: Uuid,
        media_type: &str,
        default_for: &str,
    ) -> Result<Self, AppError> {
        let collection = collections::table
            .select(Collection::as_select())
            .filter(
                collections::user_id
                    .eq(user_id)
                    .and(collections::media_type.eq(media_type))
                    .and(collections::default_for.eq(default_for)),
            )
            .first(conn)?;

        Ok(collection)
    }

    pub fn find(
        conn: &mut DbConnection,
        user_id: Uuid,
        media_type: &str,
        collection_id: &Uuid,
    ) -> Result<Self, AppError> {
        let collections = collections::table
            .filter(collections::user_id.eq(user_id))
            .filter(collections::media_type.eq(media_type))
            .filter(collections::collection_id.eq(collection_id))
            .select(Collection::as_select())
            .first(conn)?;
        Ok(collections)
    }

    pub fn find_by_media_type(
        conn: &mut DbConnection,
        user_id: Uuid,
        media_type: &str,
    ) -> Result<Vec<Self>, AppError> {
        let collections = collections::table
            .filter(collections::user_id.eq(user_id))
            .filter(collections::media_type.eq(media_type))
            .filter(collections::default_for.is_null())
            .order(collections::name.desc())
            .select(Collection::as_select())
            .load(conn)?;
        Ok(collections)
    }

    pub fn create(conn: &mut DbConnection, collection: Collection) -> Result<Self, AppError> {
        let new_collection = diesel::insert_into(collections::table)
            .values(collection)
            .get_result(conn)?;
        Ok(new_collection)
    }

    pub fn update(
        conn: &mut DbConnection,
        user_id: &Uuid,
        collection_id: &Uuid,
        collection: UpdatedCollection,
    ) -> Result<Self, AppError> {
        let updated_collection = diesel::update(collections::table)
            .filter(collections::user_id.eq(user_id))
            .filter(collections::collection_id.eq(collection_id))
            .set(collection)
            .get_result(conn)?;
        Ok(updated_collection)
    }

    pub fn delete(
        conn: &mut DbConnection,
        user_id: &Uuid,
        collection_id: &Uuid,
    ) -> Result<usize, AppError> {
        let res = diesel::delete(
            collections::table
                .filter(collections::user_id.eq(user_id))
                .filter(collections::collection_id.eq(collection_id)),
        )
        .execute(conn)?;
        Ok(res)
    }
}
