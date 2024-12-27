use actix_http::Request;
use actix_web::{
    dev::{HttpServiceFactory, Service, ServiceResponse},
    test,
    web::Data,
    App, Error,
};
use diesel::{r2d2::ConnectionManager, PgConnection};
use framerate::{db::DbConnection, tmdb};
use r2d2::{CustomizeConnection, Pool};
use std::env;

#[derive(Debug)]
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

    let database_url = format!("postgres://{db_user}:{db_password}@{db_host}:{db_port}/{db_name}");

    let manager = ConnectionManager::<DbConnection>::new(database_url);

    let pool = Pool::builder()
        .max_size(1)
        .connection_customizer(Box::new(TestConnectionCustomizer))
        .build(manager)
        .expect("Failed to create database connection pool.");

    let client = tmdb::get_client(true);
    let app = test::init_service(
        App::new()
            .app_data(Data::new(pool.clone()))
            .app_data(Data::new(client.clone()))
            .service(service),
    )
    .await;

    (app, pool)
}
