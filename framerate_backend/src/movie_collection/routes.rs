use crate::collection::{Collection, UpdatedCollection};
use crate::db::DbPool;
use crate::movie::{Movie, MOVIE_MEDIA_TYPE};
use crate::movie_entry::MovieEntry;
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
pub struct MovieCollection {
    pub collection_id: Uuid,
    pub user_id: Uuid,
    pub name: String,
    #[schema(nullable = false)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub entries: Option<Vec<MovieEntry>>,
}

impl From<Collection> for MovieCollection {
    fn from(value: Collection) -> Self {
        MovieCollection {
            collection_id: value.collection_id,
            name: value.name,
            user_id: value.user_id,
            entries: None,
        }
    }
}

impl MovieCollection {
    fn entries(mut self, entries: Vec<MovieEntry>) -> Self {
        self.entries = Some(entries);
        self
    }
}

#[derive(Debug, Deserialize, Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct NewMovieCollection {
    pub name: String,
}

#[derive(Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct SaveMovieCollectionEntryRequest {
    pub movie_id: i32,
}

#[utoipa::path(tag = "Movie Collection", responses((status = OK, body = Vec<MovieCollection>)))]
#[get("/movies/collections")]
async fn find_all(pool: web::Data<DbPool>, auth: Auth) -> actix_web::Result<impl Responder> {
    let collections = web::block(move || {
        let mut conn = pool.get()?;
        Collection::find_by_media_type(&mut conn, auth.user_id, MOVIE_MEDIA_TYPE)
    })
    .await??;

    let collections = collections
        .into_iter()
        .map(MovieCollection::from)
        .collect::<Vec<MovieCollection>>();

    Ok(Success::new(collections))
}

#[utoipa::path(tag = "Movie Collection", responses((status = OK, body = MovieCollection)))]
#[get("/movies/collections/{collection_id}")]
async fn find(
    pool: web::Data<DbPool>,
    auth: Auth,
    collection_id: web::Path<Uuid>,
) -> actix_web::Result<impl Responder> {
    let movie_collection = web::block(move || {
        let mut conn = pool.get()?;
        let collection =
            Collection::find(&mut conn, auth.user_id, MOVIE_MEDIA_TYPE, &collection_id)?;
        let entries = MovieEntry::find_all(&mut conn, auth.user_id, collection.collection_id)?;
        Ok::<MovieCollection, AppError>(MovieCollection::from(collection).entries(entries))
    })
    .await??;

    Ok(Success::new(movie_collection))
}

#[utoipa::path(tag = "Movie Collection", responses((status = OK, body = MovieCollection)))]
#[post("/movies/collections")]
async fn create(
    pool: web::Data<DbPool>,
    auth: Auth,
    params: web::Json<NewMovieCollection>,
) -> actix_web::Result<impl Responder> {
    let params = params.into_inner();

    let collection = Collection {
        collection_id: Uuid::new_v4(),
        user_id: auth.user_id,
        name: params.name,
        media_type: MOVIE_MEDIA_TYPE.to_string(),
        default_for: None,
    };

    let collection = web::block(move || {
        let mut conn = pool.get()?;
        Collection::create(&mut conn, collection)
    })
    .await??;

    Ok(Success::new(MovieCollection::from(collection)))
}

#[utoipa::path(tag = "Movie Collection", responses((status = OK, body = MovieCollection)))]
#[put("/movies/collections/{collection_id}")]
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

    Ok(Success::new(MovieCollection::from(collection)))
}

#[utoipa::path(tag = "Movie Collection", responses((status = OK, body = DeleteResponse)))]
#[delete("/movies/collections/{collection_id}")]
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

#[utoipa::path(tag = "Movie Collection", responses((status = OK, body = MovieEntry)))]
#[post("/movies/collections/{collection_id}")]
async fn create_entry(
    pool: web::Data<DbPool>,
    client: web::Data<TmdbClient>,
    auth: Auth,
    collection_id: web::Path<Uuid>,
    movie_entry: web::Json<SaveMovieCollectionEntryRequest>,
) -> actix_web::Result<impl Responder> {
    let movie = Movie::find(&client, &movie_entry.movie_id).await?;

    let movie_entry = web::block(move || {
        let mut conn = pool.get()?;
        let collection =
            Collection::find(&mut conn, auth.user_id, MOVIE_MEDIA_TYPE, &collection_id)?;

        let movie_entry_to_save = MovieEntry {
            collection_id: collection.collection_id,
            user_id: auth.user_id,
            movie_id: movie_entry.movie_id,
            imdb_id: movie.imdb_id,
            title: movie.title,
            poster_path: movie.poster_path,
            release_date: movie.release_date,
            status: movie.status,
            updated_at: Utc::now().naive_utc().date(),
        };

        MovieEntry::create(&mut conn, movie_entry_to_save)
    })
    .await??;

    Ok(Success::new(movie_entry))
}

#[utoipa::path(tag = "Movie Collection", responses((status = OK, body = DeleteResponse)))]
#[delete("/movies/collections/{collection_id}/{movie_id}")]
async fn delete_entry(
    pool: web::Data<DbPool>,
    auth: Auth,
    path: web::Path<(Uuid, i32)>,
) -> actix_web::Result<impl Responder> {
    let (collection_id, movie_id) = path.into_inner();

    let count = web::block(move || {
        let mut conn = pool.get()?;
        let collection =
            Collection::find(&mut conn, auth.user_id, MOVIE_MEDIA_TYPE, &collection_id)?;
        MovieEntry::delete(&mut conn, collection.collection_id, movie_id)
    })
    .await??;

    if count == 0 {
        return Err(AppError::external(404, "Collection entry not found"))?;
    }

    Ok(Success::new(DeleteResponse { count }))
}

#[utoipa::path(tag = "Movie Collection", responses((status = OK, body = Vec<Uuid>)))]
#[get("/movies/collections/movie/{movie_id}")]
async fn find_by_movie(
    pool: web::Data<DbPool>,
    auth: Auth,
    movie_id: web::Path<i32>,
) -> actix_web::Result<impl Responder> {
    let collections = web::block(move || {
        let mut conn = pool.get()?;
        MovieEntry::find_collections(&mut conn, auth.user_id, &movie_id)
    })
    .await??;

    Ok(Success::new(collections))
}
