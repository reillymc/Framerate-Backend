use crate::collection::Collection;
use crate::db::DbPool;
use crate::movie::{Movie, MOVIE_MEDIA_TYPE};
use crate::movie_entry::MovieEntry;
use crate::tmdb::TmdbClient;
use crate::utils::response_body::{DeleteResponse, Success};
use crate::utils::{jwt::Auth, AppError};
use actix_web::{delete, Responder};
use actix_web::{get, post, web};
use chrono::{NaiveDate, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

const DEFAULT_WATCHLIST: &str = "watchlist";

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MovieWatchlistEntry {
    pub collection_id: Uuid,
    pub movie_id: i32,
    pub user_id: Uuid,
    pub title: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub imdb_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub poster_path: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub release_date: Option<NaiveDate>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub status: Option<String>,
    pub updated_at: NaiveDate,
}

impl From<MovieEntry> for MovieWatchlistEntry {
    fn from(value: MovieEntry) -> Self {
        MovieWatchlistEntry {
            collection_id: value.collection_id,
            movie_id: value.movie_id,
            user_id: value.user_id,
            imdb_id: value.imdb_id,
            title: value.title,
            poster_path: value.poster_path,
            release_date: value.release_date,
            status: value.status,
            updated_at: value.updated_at,
        }
    }
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MovieWatchlist {
    pub name: String,
    pub entries: Option<Vec<MovieWatchlistEntry>>,
}

impl MovieWatchlist {
    fn entries(mut self, entries: Vec<MovieEntry>) -> Self {
        self.entries = Some(entries.into_iter().map(MovieWatchlistEntry::from).collect());
        self
    }
}

impl From<Collection> for MovieWatchlist {
    fn from(value: Collection) -> Self {
        MovieWatchlist {
            name: value.name,
            entries: None,
        }
    }
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SaveMovieWatchlistEntryRequest {
    pub movie_id: i32,
}

#[get("/movies/watchlist")]
async fn find(pool: web::Data<DbPool>, auth: Auth) -> actix_web::Result<impl Responder> {
    let watchlist = web::block(move || {
        let mut conn = pool.get()?;
        let watchlist =
            Collection::find_default(&mut conn, auth.user_id, MOVIE_MEDIA_TYPE, DEFAULT_WATCHLIST);

        let watchlist = match watchlist {
            Ok(watchlist) => Ok(watchlist),
            _ => Collection::create(
                &mut conn,
                Collection {
                    collection_id: Uuid::new_v4(),
                    media_type: MOVIE_MEDIA_TYPE.to_string(),
                    user_id: auth.user_id,
                    name: "Movie Watchlist".to_string(),
                    default_for: Some(DEFAULT_WATCHLIST.to_string()),
                },
            ),
        }?;

        let entries = MovieEntry::find_all(&mut conn, auth.user_id, watchlist.collection_id)?;

        Ok::<MovieWatchlist, AppError>(MovieWatchlist::from(watchlist).entries(entries))
    })
    .await??;

    Ok(Success::new(watchlist))
}

#[get("/movies/watchlist/{movie_id}")]
async fn find_entry(
    pool: web::Data<DbPool>,
    auth: Auth,
    path: web::Path<i32>,
) -> actix_web::Result<impl Responder> {
    let movie_id = path.into_inner();

    let movie_entry = web::block(move || {
        let mut conn = pool.get()?;
        let collection =
            Collection::find_default(&mut conn, auth.user_id, MOVIE_MEDIA_TYPE, DEFAULT_WATCHLIST)?;
        MovieEntry::find(&mut conn, auth.user_id, collection.collection_id, movie_id)
    })
    .await??;

    Ok(Success::new(MovieWatchlistEntry::from(movie_entry)))
}

#[post("/movies/watchlist")]
async fn create_entry(
    pool: web::Data<DbPool>,
    client: web::Data<TmdbClient>,
    auth: Auth,
    movie_entry: web::Json<SaveMovieWatchlistEntryRequest>,
) -> actix_web::Result<impl Responder> {
    let movie = Movie::find(&client, &movie_entry.movie_id).await?;

    let movie_entry = web::block(move || {
        let mut conn = pool.get()?;
        let collection =
            Collection::find_default(&mut conn, auth.user_id, MOVIE_MEDIA_TYPE, DEFAULT_WATCHLIST)?;

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

    Ok(Success::new(MovieWatchlistEntry::from(movie_entry)))
}

#[delete("/movies/watchlist/{movie_id}")]
async fn delete_entry(
    pool: web::Data<DbPool>,
    auth: Auth,
    path: web::Path<i32>,
) -> actix_web::Result<impl Responder> {
    let movie_id = path.into_inner();

    let count = web::block(move || {
        let mut conn = pool.get()?;
        let collection =
            Collection::find_default(&mut conn, auth.user_id, MOVIE_MEDIA_TYPE, DEFAULT_WATCHLIST)?;
        MovieEntry::delete(&mut conn, collection.collection_id, movie_id)
    })
    .await??;

    if count == 0 {
        return Err(AppError::external(404, "Watchlist entry not found"))?;
    }

    Ok(Success::new(DeleteResponse { count }))
}
