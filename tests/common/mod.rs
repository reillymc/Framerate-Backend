pub mod setup {
    use actix_http::Request;
    use actix_web::{
        dev::{HttpServiceFactory, Service, ServiceResponse},
        test,
        web::Data,
        App, Error,
    };
    use diesel::{r2d2::ConnectionManager, PgConnection};
    use framerate::db::DbConnection;
    use r2d2::{CustomizeConnection, Pool};
    use std::env;

    #[derive(Debug, Clone, Copy)]
    struct TestConnectionCustomizer;

    impl<C, E> CustomizeConnection<C, E> for TestConnectionCustomizer
    where
        C: diesel::Connection,
    {
        fn on_acquire(&self, conn: &mut C) -> Result<(), E> {
            conn.begin_test_transaction()
                .expect("Failed to start test transaction");

            Ok(())
        }
    }

    pub async fn create_app<F>(
        service: F,
    ) -> (
        impl Service<Request, Response = ServiceResponse, Error = Error>,
        Pool<ConnectionManager<PgConnection>>,
    )
    where
        F: HttpServiceFactory + 'static,
    {
        let db_user = env::var("POSTGRES_USER").expect("POSTGRES_USER must be set");
        let db_password = env::var("POSTGRES_PASSWORD").expect("POSTGRES_PASSWORD must be set");
        let db_name = env::var("POSTGRES_DB").expect("POSTGRES_DB must be set");
        let db_port = env::var("PGPORT").expect("PGPORT must be set");

        let db_host = env::var("TEST_POSTGRES_HOST").expect("TEST_POSTGRES_HOST must be set");

        let database_url =
            format!("postgres://{db_user}:{db_password}@{db_host}:{db_port}/{db_name}");

        let manager = ConnectionManager::<DbConnection>::new(database_url);

        let pool = Pool::builder()
            .max_size(1)
            .connection_customizer(Box::new(TestConnectionCustomizer))
            .build(manager)
            .expect("Failed to create database connection pool.");
        let app = test::init_service(
            App::new()
                .app_data(Data::new(pool.clone()))
                .service(service),
        )
        .await;

        (app, pool)
    }
}

pub mod data {
    use chrono::{NaiveDate, Utc};
    use diesel::{r2d2::ConnectionManager, PgConnection};
    use r2d2::PooledConnection;
    use rand::Rng;
    use uuid::Uuid;

    use framerate::{
        movie::Movie,
        movie_review::{MovieReview, SaveMovieReviewRequest},
        review::Review,
        user::{self, NewUser, User},
        utils::jwt::create_token,
        watchlist::{NewWatchlist, Watchlist},
    };

    pub fn create_authed_user(
        conn: &mut PooledConnection<ConnectionManager<PgConnection>>,
    ) -> (String, User) {
        let new_user = NewUser {
            first_name: Uuid::new_v4().to_string(),
            last_name: Uuid::new_v4().to_string(),
            avatar_uri: Some(Uuid::new_v4().to_string()),
            email: Some(Uuid::new_v4().to_string()),
            password: Some(Uuid::new_v4().to_string()),
            configuration: None,
            user_id: None,
        };

        let mut user = user::User::create(conn, new_user.clone()).unwrap();

        user.password = new_user.password;

        let token = create_token(user.user_id, &user.email.clone().unwrap()).unwrap();

        (token, user)
    }

    pub fn create_user(conn: &mut PooledConnection<ConnectionManager<PgConnection>>) -> User {
        user::User::create(
            conn,
            NewUser {
                first_name: Uuid::new_v4().to_string(),
                last_name: Uuid::new_v4().to_string(),
                avatar_uri: Some(Uuid::new_v4().to_string()),
                email: Some(Uuid::new_v4().to_string()),
                password: Some(Uuid::new_v4().to_string()),
                configuration: None,
                user_id: None,
            },
        )
        .unwrap()
    }

    pub fn generate_review(user_id: Uuid) -> Review {
        let mut rng = rand::thread_rng();

        Review {
            review_id: Uuid::new_v4(),
            user_id,
            title: Some(Uuid::new_v4().to_string()),
            date: Some(Utc::now().naive_utc().date()),
            rating: rng.gen(),
            description: Some(Uuid::new_v4().to_string()),
            venue: Some(Uuid::new_v4().to_string()),
        }
    }

    pub fn generate_sample_movie() -> Movie {
        Movie {
        id: 4638,
        imdb_id: Some("tt0425112".to_string()),
        title: "Hot Fuzz".to_string(),
        poster_path: Some("/1ub4urtlb2Re27Qw0lBcc1kt2pw.jpg".to_string()),
        backdrop_path: Some("/e1rPzkIcBEJiAd3piGirt7qVux7.jpg".to_string()),
        release_date: NaiveDate::from_ymd_opt(2007, 5, 20),
        overview: Some("Former London constable Nicholas Angel finds it difficult to adapt to his new assignment in the sleepy British village of Sandford. Not only does he miss the excitement of the big city, but he also has a well-meaning oaf for a partner. However, when a series of grisly accidents rocks Sandford, Angel smells something rotten in the idyllic village.".to_string()),
        tagline: Some("Big cops. Small town. Moderate violence.".to_string()),
        popularity: Some(26.13),
        runtime: Some(121)
        }
    }

    pub fn generate_movie_review(user_id: Uuid, review_id: Uuid) -> MovieReview {
        MovieReview {
            review_id,
            user_id,
            imdb_id: Some(Uuid::new_v4().to_string()),
            movie_id: generate_sample_movie().id,
            title: Uuid::new_v4().to_string(),
            poster_path: Some(Uuid::new_v4().to_string()),
            release_date: Some(Utc::now().naive_utc().date()),
        }
    }

    pub fn generate_save_movie_review() -> SaveMovieReviewRequest {
        let mut rng = rand::thread_rng();

        SaveMovieReviewRequest {
            title: Some(Uuid::new_v4().to_string()),
            date: Some(Utc::now().naive_utc().date()),
            rating: rng.gen(),
            description: Some(Uuid::new_v4().to_string()),
            venue: Some(Uuid::new_v4().to_string()),
            company: None,
        }
    }

    pub fn generate_watchlist(user_id: Uuid) -> Watchlist {
        let mut rng = rand::thread_rng();
        let media_type = if rng.gen() {
            "movie".to_string()
        } else {
            "show".to_string()
        };
        Watchlist {
            watchlist_id: Uuid::new_v4(),
            user_id,
            name: Uuid::new_v4().to_string(),
            media_type,
        }
    }

    pub fn generate_new_watchlist() -> NewWatchlist {
        let mut rng = rand::thread_rng();
        let media_type = if rng.gen() {
            "movie".to_string()
        } else {
            "show".to_string()
        };

        NewWatchlist {
            name: Uuid::new_v4().to_string(),
            media_type,
        }
    }
}

pub mod process {
    use actix_web::dev::ServiceResponse;
    use actix_web::test;
    use framerate::utils::response_body::Success;
    use serde::Deserialize;

    pub async fn parse_body<T: for<'a> Deserialize<'a>>(response: ServiceResponse) -> Success<T> {
        let body = test::read_body(response).await;
        // println!("{:?}", body);
        let data: Success<T> = serde_json::from_slice(&body).unwrap();
        data
    }
}
