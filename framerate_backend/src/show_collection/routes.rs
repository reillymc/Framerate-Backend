use crate::collection::{Collection, UpdatedCollection};
use crate::db::DbPool;
use crate::show::{Show, SHOW_MEDIA_TYPE};
use crate::show_entry::ShowEntry;
use crate::tmdb::TmdbClient;
use crate::utils::response_body::{DeleteResponse, Success};
use crate::utils::{jwt::Auth, AppError};
use actix_web::{delete, put, Responder};
use actix_web::{get, post, web};
use chrono::Utc;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::Uuid;

#[derive(Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct ShowCollection {
    pub collection_id: Uuid,
    pub user_id: Uuid,
    pub name: String,
    #[schema(nullable = false)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub entries: Option<Vec<ShowEntry>>,
}

impl From<Collection> for ShowCollection {
    fn from(value: Collection) -> Self {
        ShowCollection {
            collection_id: value.collection_id,
            name: value.name,
            user_id: value.user_id,
            entries: None,
        }
    }
}

impl ShowCollection {
    fn entries(mut self, entries: Vec<ShowEntry>) -> Self {
        self.entries = Some(entries);
        self
    }
}

#[derive(Debug, Deserialize, Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct NewShowCollection {
    pub name: String,
}

#[derive(Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct SaveShowCollectionEntryRequest {
    pub show_id: i32,
}

#[utoipa::path(tag = "Show Collection", responses((status = OK, body = Vec<ShowCollection>)))]
#[get("/shows/collections")]
async fn find_all(pool: web::Data<DbPool>, auth: Auth) -> actix_web::Result<impl Responder> {
    let collections = web::block(move || {
        let mut conn = pool.get()?;
        Collection::find_by_media_type(&mut conn, auth.user_id, SHOW_MEDIA_TYPE)
    })
    .await??;

    let collections = collections
        .into_iter()
        .map(ShowCollection::from)
        .collect::<Vec<ShowCollection>>();

    Ok(Success::new(collections))
}

#[utoipa::path(tag = "Show Collection", responses((status = OK, body = ShowCollection)))]
#[get("/shows/collections/{collection_id}")]
async fn find(
    pool: web::Data<DbPool>,
    auth: Auth,
    collection_id: web::Path<Uuid>,
) -> actix_web::Result<impl Responder> {
    let show_collection = web::block(move || {
        let mut conn = pool.get()?;
        let collection =
            Collection::find(&mut conn, auth.user_id, SHOW_MEDIA_TYPE, &collection_id)?;
        let entries = ShowEntry::find_all(&mut conn, auth.user_id, collection.collection_id)?;
        Ok::<ShowCollection, AppError>(ShowCollection::from(collection).entries(entries))
    })
    .await??;

    Ok(Success::new(show_collection))
}

#[utoipa::path(tag = "Show Collection", responses((status = OK, body = ShowCollection)))]
#[post("/shows/collections")]
async fn create(
    pool: web::Data<DbPool>,
    auth: Auth,
    params: web::Json<NewShowCollection>,
) -> actix_web::Result<impl Responder> {
    let params = params.into_inner();

    let collection = Collection {
        collection_id: Uuid::new_v4(),
        user_id: auth.user_id,
        name: params.name,
        media_type: SHOW_MEDIA_TYPE.to_string(),
        default_for: None,
    };

    let collection = web::block(move || {
        let mut conn = pool.get()?;
        Collection::create(&mut conn, collection)
    })
    .await??;

    Ok(Success::new(ShowCollection::from(collection)))
}

#[utoipa::path(tag = "Show Collection", responses((status = OK, body = ShowCollection)))]
#[put("/shows/collections/{collection_id}")]
async fn update(
    pool: web::Data<DbPool>,
    auth: Auth,
    collection_id: web::Path<Uuid>,
    params: web::Json<UpdatedCollection>,
) -> actix_web::Result<impl Responder> {
    let params = params.into_inner();

    let collection = web::block(move || {
        let mut conn = pool.get()?;
        Collection::update(&mut conn, &auth.user_id, &collection_id, params)
    })
    .await??;

    Ok(Success::new(ShowCollection::from(collection)))
}

#[utoipa::path(tag = "Show Collection", responses((status = OK, body = DeleteResponse)))]
#[delete("/shows/collections/{collection_id}")]
async fn delete(
    pool: web::Data<DbPool>,
    auth: Auth,
    collection_id: web::Path<Uuid>,
) -> actix_web::Result<impl Responder> {
    let count = web::block(move || {
        let mut conn = pool.get()?;
        Collection::delete(&mut conn, &auth.user_id, &collection_id)
    })
    .await??;

    if count == 0 {
        return Err(AppError::external(404, "Collection entry not found"))?;
    }

    Ok(Success::new(DeleteResponse { count }))
}

#[utoipa::path(tag = "Show Collection", responses((status = OK, body = ShowEntry)))]
#[post("/shows/collections/{collection_id}")]
async fn create_entry(
    pool: web::Data<DbPool>,
    client: web::Data<TmdbClient>,
    auth: Auth,
    collection_id: web::Path<Uuid>,
    show_entry: web::Json<SaveShowCollectionEntryRequest>,
) -> actix_web::Result<impl Responder> {
    let show = Show::find(&client, &show_entry.show_id).await?;

    let show_entry = web::block(move || {
        let mut conn = pool.get()?;
        let collection =
            Collection::find(&mut conn, auth.user_id, SHOW_MEDIA_TYPE, &collection_id)?;

        let imdb_id = if let Some(external_ids) = show.external_ids {
            external_ids.imdb_id
        } else {
            None
        };

        let show_entry_to_save = ShowEntry {
            collection_id: collection.collection_id,
            user_id: auth.user_id,
            show_id: show_entry.show_id,
            imdb_id,
            name: show.name,
            poster_path: show.poster_path,
            first_air_date: show.first_air_date,
            last_air_date: show.last_air_date,
            next_air_date: show.next_air_date,
            status: show.status,
            updated_at: Utc::now().naive_utc().date(),
        };

        ShowEntry::create(&mut conn, show_entry_to_save)
    })
    .await??;

    Ok(Success::new(show_entry))
}

#[utoipa::path(tag = "Show Collection", responses((status = OK, body = DeleteResponse)))]
#[delete("/shows/collections/{collection_id}/{show_id}")]
async fn delete_entry(
    pool: web::Data<DbPool>,
    auth: Auth,
    path: web::Path<(Uuid, i32)>,
) -> actix_web::Result<impl Responder> {
    let (collection_id, show_id) = path.into_inner();

    let count = web::block(move || {
        let mut conn = pool.get()?;
        let collection =
            Collection::find(&mut conn, auth.user_id, SHOW_MEDIA_TYPE, &collection_id)?;
        ShowEntry::delete(&mut conn, collection.collection_id, show_id)
    })
    .await??;

    if count == 0 {
        return Err(AppError::external(404, "Collection entry not found"))?;
    }

    Ok(Success::new(DeleteResponse { count }))
}

#[utoipa::path(tag = "Show Collection", responses((status = OK, body = Vec<Uuid>)))]
#[get("/shows/collections/show/{show_id}")]
async fn find_by_show(
    pool: web::Data<DbPool>,
    auth: Auth,
    show_id: web::Path<i32>,
) -> actix_web::Result<impl Responder> {
    let collections = web::block(move || {
        let mut conn = pool.get()?;
        ShowEntry::find_collections(&mut conn, auth.user_id, &show_id)
    })
    .await??;

    Ok(Success::new(collections))
}
