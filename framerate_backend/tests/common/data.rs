use chrono::{NaiveDate, Utc};
use diesel::{r2d2::ConnectionManager, PgConnection};
use r2d2::PooledConnection;
use rand::Rng;
use uuid::Uuid;

use framerate::{
    collection::{Collection, UpdatedCollection},
    company::{Company, SaveCompany},
    movie::{Movie, MOVIE_MEDIA_TYPE},
    movie_collection::{NewMovieCollection, SaveMovieCollectionEntryRequest},
    movie_entry::MovieEntry,
    movie_review::{MovieReview, SaveMovieReviewRequest},
    movie_watchlist::SaveMovieWatchlistEntryRequest,
    review::Review,
    season::Season,
    season_review::{SaveSeasonReviewRequest, SeasonReview},
    show::{ExternalIds, Show, SHOW_MEDIA_TYPE},
    show_collection::{NewShowCollection, SaveShowCollectionEntryRequest},
    show_entry::ShowEntry,
    show_review::{SaveShowReviewRequest, ShowReview},
    show_watchlist::SaveShowWatchlistEntryRequest,
    user::{NewUser, PermissionLevel, User},
    utils::jwt::create_token,
};

pub fn create_authed_user(
    conn: &mut PooledConnection<ConnectionManager<PgConnection>>,
) -> (String, User) {
    let new_user = generate_save_new_user();

    let mut user = User::create(conn, new_user.clone()).unwrap();

    user.password = Some(new_user.password);

    let token = create_token(user.user_id, PermissionLevel::GeneralUser).unwrap();

    (token, user)
}

pub fn create_authed_admin_user(
    conn: &mut PooledConnection<ConnectionManager<PgConnection>>,
) -> (String, User) {
    let new_user = generate_save_new_user();

    let mut user = User::create_admin(conn, new_user.clone()).unwrap();

    user.password = Some(new_user.password);

    let token = create_token(user.user_id, PermissionLevel::AdminUser).unwrap();

    (token, user)
}

// Create in DB

pub fn create_user(conn: &mut PooledConnection<ConnectionManager<PgConnection>>) -> User {
    User::create(conn, generate_save_new_user()).unwrap()
}

pub fn create_default_show_watchlist(
    conn: &mut PooledConnection<ConnectionManager<PgConnection>>,
    user: &User,
) -> Collection {
    Collection::create(conn, generate_default_show_watchlist(user.user_id)).unwrap()
}

pub fn create_default_movie_watchlist(
    conn: &mut PooledConnection<ConnectionManager<PgConnection>>,
    user: &User,
) -> Collection {
    Collection::create(conn, generate_default_movie_watchlist(user.user_id)).unwrap()
}

pub fn create_movie_entry(
    conn: &mut PooledConnection<ConnectionManager<PgConnection>>,
    user: &User,
    collection: &Collection,
) -> MovieEntry {
    MovieEntry::create(
        conn,
        generate_movie_entry(user.user_id, collection.collection_id),
    )
    .unwrap()
}

pub fn create_movie_collection(
    conn: &mut PooledConnection<ConnectionManager<PgConnection>>,
    user: &User,
) -> Collection {
    Collection::create(conn, generate_movie_collection(user.user_id)).unwrap()
}
pub fn create_show_collection(
    conn: &mut PooledConnection<ConnectionManager<PgConnection>>,
    user: &User,
) -> Collection {
    Collection::create(conn, generate_show_collection(user.user_id)).unwrap()
}

pub fn create_show_entry(
    conn: &mut PooledConnection<ConnectionManager<PgConnection>>,
    user: &User,
    collection: &Collection,
) -> ShowEntry {
    ShowEntry::create(
        conn,
        generate_show_entry(user.user_id, collection.collection_id),
    )
    .unwrap()
}

pub fn create_review(
    conn: &mut PooledConnection<ConnectionManager<PgConnection>>,
    user: &User,
) -> Review {
    Review::create(conn, generate_review(user.user_id)).unwrap()
}

pub fn create_movie_review(
    conn: &mut PooledConnection<ConnectionManager<PgConnection>>,
    user: &User,
    review: &Review,
) -> MovieReview {
    MovieReview::create(conn, generate_movie_review(user.user_id, review.review_id)).unwrap()
}

pub fn create_show_review(
    conn: &mut PooledConnection<ConnectionManager<PgConnection>>,
    user: &User,
    review: &Review,
) -> ShowReview {
    ShowReview::create(conn, generate_show_review(user.user_id, review.review_id)).unwrap()
}

pub fn create_season_review(
    conn: &mut PooledConnection<ConnectionManager<PgConnection>>,
    user: &User,
    review: &Review,
) -> SeasonReview {
    SeasonReview::create(conn, generate_season_review(user.user_id, review.review_id)).unwrap()
}

pub fn create_company(
    conn: &mut PooledConnection<ConnectionManager<PgConnection>>,
    user: &User,
) -> Company {
    Company::create(conn, generate_save_company(), user.user_id).unwrap()
}

// Generate save request items

pub fn generate_save_movie_review() -> SaveMovieReviewRequest {
    let mut rng = rand::thread_rng();

    SaveMovieReviewRequest {
        title: Some(Uuid::new_v4().to_string()),
        date: Some(Utc::now().naive_utc().date()),
        rating: Some(rng.gen_range(0..101)),
        description: Some(Uuid::new_v4().to_string()),
        venue: Some(Uuid::new_v4().to_string()),
        company: None,
    }
}

pub fn generate_save_new_user() -> NewUser {
    NewUser {
        avatar_uri: Some(Uuid::new_v4().to_string()),
        configuration: None,
        email: Uuid::new_v4().to_string(),
        first_name: Uuid::new_v4().to_string(),
        last_name: Uuid::new_v4().to_string(),
        password: Uuid::new_v4().to_string(),
        is_admin: Some(false),
    }
}

pub fn generate_save_show_review() -> SaveShowReviewRequest {
    let mut rng = rand::thread_rng();

    SaveShowReviewRequest {
        title: Some(Uuid::new_v4().to_string()),
        date: Some(Utc::now().naive_utc().date()),
        rating: Some(rng.gen_range(0..101)),
        description: Some(Uuid::new_v4().to_string()),
        venue: Some(Uuid::new_v4().to_string()),
        company: None,
    }
}

pub fn generate_save_season_review() -> SaveSeasonReviewRequest {
    let mut rng = rand::thread_rng();

    SaveSeasonReviewRequest {
        title: Some(Uuid::new_v4().to_string()),
        date: Some(Utc::now().naive_utc().date()),
        rating: Some(rng.gen_range(0..101)),
        description: Some(Uuid::new_v4().to_string()),
        venue: Some(Uuid::new_v4().to_string()),
        company: None,
    }
}

pub fn generate_save_movie_collection() -> NewMovieCollection {
    NewMovieCollection {
        name: Uuid::new_v4().to_string(),
    }
}

pub fn generate_save_show_collection() -> NewShowCollection {
    NewShowCollection {
        name: Uuid::new_v4().to_string(),
    }
}

pub fn generate_update_movie_collection() -> UpdatedCollection {
    UpdatedCollection {
        name: Uuid::new_v4().to_string(),
    }
}

pub fn generate_update_show_collection() -> UpdatedCollection {
    UpdatedCollection {
        name: Uuid::new_v4().to_string(),
    }
}

pub fn generate_save_movie_collection_entry() -> SaveMovieCollectionEntryRequest {
    let movie = generate_sample_movie();
    SaveMovieCollectionEntryRequest { movie_id: movie.id }
}

pub fn generate_save_show_collection_entry() -> SaveShowCollectionEntryRequest {
    let show = generate_sample_show();
    SaveShowCollectionEntryRequest { show_id: show.id }
}

pub fn generate_save_show_watchlist_entry() -> SaveShowWatchlistEntryRequest {
    let show = generate_sample_show();
    SaveShowWatchlistEntryRequest { show_id: show.id }
}

pub fn generate_save_company() -> SaveCompany {
    SaveCompany {
        first_name: Uuid::new_v4().to_string(),
        last_name: Uuid::new_v4().to_string(),
    }
}

pub fn generate_save_movie_watchlist_entry() -> SaveMovieWatchlistEntryRequest {
    let movie = generate_sample_movie();
    SaveMovieWatchlistEntryRequest { movie_id: movie.id }
}

// Generate sample data

pub fn generate_sample_movie() -> Movie {
    Movie {
        id: 4638,
        imdb_id: Some("tt0425112".to_string()),
        title: "Hot Fuzz".to_string(),
        poster_path: Some("/1ub4urtlb2Re27Qw0lBcc1kt2pw.jpg".to_string()),
        backdrop_path: Some("/e1rPzkIcBEJiAd3piGirt7qVux7.jpg".to_string()),
        release_date: NaiveDate::from_ymd_opt(2007, 5, 20),
        overview: Some("Former London constable Nicholas Angel finds it difficult to adapt to his new assignment in...".to_string()),
        tagline: Some("Big cops. Small town. Moderate violence.".to_string()),
        popularity: Some(26.13),
        runtime: Some(121),
        status: Some("Released".to_string()),
        credits: None
    }
}

pub fn generate_sample_show() -> Show {
    Show {
        id: 57243,
        name: "Doctor Who".to_string(),
        poster_path: Some("/4edFyasCrkH4MKs6H4mHqlrxA6b.jpg".to_string()),
        backdrop_path: Some("/vcFW09U4834DyFOeRZpsx9x1D3S.jpg".to_string()),
        first_air_date: NaiveDate::from_ymd_opt(2005, 3, 26),
        last_air_date: NaiveDate::from_ymd_opt(2021, 12, 05),
        status: Some("Ended".to_string()),
        overview: Some(
            "The Doctor is a Time Lord: a 900 year old alien with 2 hearts, part of a gifted..."
                .to_string(),
        ),
        tagline: Some("Space. For all.".to_string()),
        popularity: Some(361.611),
        external_ids: Some(ExternalIds {
            imdb_id: Some("tt0436992".to_string()),
            tvdb_id: Some(78804),
        }),
        next_air_date: None,
        seasons: None,
        credits: None,
    }
}

pub fn generate_sample_season() -> Season {
    Season {
        show_id: 57243,
        season_number: 1,
        name: Some("Series 1".to_string()),
        poster_path: Some("/9Jt2FFCAME7eHDC28r4qCHErhhF.jpg".to_string()),
        overview: Some(
            "The first series features Christopher Eccleston as the ninth incarnation of the..."
                .to_string(),
        ),
        air_date: NaiveDate::from_ymd_opt(2005, 3, 26),
        episode_count: None,
        episodes: None,
    }
}

pub fn generate_sample_media_type() -> String {
    let mut rng = rand::thread_rng();
    let media_type = if rng.gen() {
        MOVIE_MEDIA_TYPE.to_string()
    } else {
        SHOW_MEDIA_TYPE.to_string()
    };

    media_type
}

// Helpers

fn generate_review(user_id: Uuid) -> Review {
    let mut rng = rand::thread_rng();

    Review {
        review_id: Uuid::new_v4(),
        user_id,
        title: Some(Uuid::new_v4().to_string()),
        date: Some(Utc::now().naive_utc().date()),
        rating: Some(rng.gen_range(0..101)),
        description: Some(Uuid::new_v4().to_string()),
        venue: Some(Uuid::new_v4().to_string()),
    }
}

fn generate_movie_review(user_id: Uuid, review_id: Uuid) -> MovieReview {
    let movie = generate_sample_movie();

    MovieReview {
        review_id,
        user_id,
        imdb_id: movie.imdb_id,
        movie_id: movie.id,
        title: movie.title,
        poster_path: movie.poster_path,
        release_date: movie.release_date,
    }
}

fn generate_movie_collection(user_id: Uuid) -> Collection {
    Collection {
        collection_id: Uuid::new_v4(),
        user_id,
        name: Uuid::new_v4().to_string(),
        media_type: MOVIE_MEDIA_TYPE.to_string(),
        default_for: None,
    }
}

fn generate_show_collection(user_id: Uuid) -> Collection {
    Collection {
        collection_id: Uuid::new_v4(),
        user_id,
        name: Uuid::new_v4().to_string(),
        media_type: SHOW_MEDIA_TYPE.to_string(),
        default_for: None,
    }
}

fn generate_default_show_watchlist(user_id: Uuid) -> Collection {
    Collection {
        collection_id: Uuid::new_v4(),
        user_id,
        name: Uuid::new_v4().to_string(),
        media_type: SHOW_MEDIA_TYPE.to_string(),
        default_for: Some("watchlist".to_string()),
    }
}

fn generate_default_movie_watchlist(user_id: Uuid) -> Collection {
    Collection {
        collection_id: Uuid::new_v4(),
        user_id,
        name: Uuid::new_v4().to_string(),
        media_type: MOVIE_MEDIA_TYPE.to_string(),
        default_for: Some("watchlist".to_string()),
    }
}

fn generate_movie_entry(user_id: Uuid, collection_id: Uuid) -> MovieEntry {
    let movie = generate_sample_movie();

    MovieEntry {
        user_id,
        collection_id,
        imdb_id: movie.imdb_id,
        movie_id: movie.id,
        title: movie.title,
        poster_path: movie.poster_path,
        release_date: movie.release_date,
        status: movie.status,
        updated_at: Utc::now().naive_utc().date(),
    }
}

fn generate_show_entry(user_id: Uuid, collection_id: Uuid) -> ShowEntry {
    let show = generate_sample_show();

    ShowEntry {
        user_id,
        collection_id,
        imdb_id: show.external_ids.unwrap().imdb_id,
        show_id: show.id,
        name: show.name,
        poster_path: show.poster_path,
        first_air_date: show.first_air_date,
        last_air_date: show.last_air_date,
        next_air_date: show.next_air_date,
        status: show.status,
        updated_at: Utc::now().naive_utc().date(),
    }
}

fn generate_show_review(user_id: Uuid, review_id: Uuid) -> ShowReview {
    let show = generate_sample_show();

    ShowReview {
        review_id,
        user_id,
        imdb_id: show.external_ids.unwrap().imdb_id,
        show_id: show.id,
        name: show.name,
        poster_path: show.poster_path,
        first_air_date: show.first_air_date,
    }
}

fn generate_season_review(user_id: Uuid, review_id: Uuid) -> SeasonReview {
    let season = generate_sample_season();

    SeasonReview {
        review_id,
        user_id,
        show_id: season.show_id,
        season_number: season.season_number,
        name: season.name,
        poster_path: season.poster_path,
        air_date: season.air_date,
    }
}
