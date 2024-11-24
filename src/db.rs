use diesel::pg;
use diesel::prelude::*;
use diesel_migrations::{embed_migrations, EmbeddedMigrations, MigrationHarness};
use std::env;
use tracing::info;

const MIGRATIONS: EmbeddedMigrations = embed_migrations!("migrations");
pub fn establish_connection() -> PgConnection {
    let db_user = env::var("POSTGRES_USER").expect("POSTGRES_USER must be set");
    let db_password = env::var("POSTGRES_PASSWORD").expect("POSTGRES_PASSWORD must be set");
    let db_name = env::var("POSTGRES_DB").expect("POSTGRES_DB must be set");
    let db_host = env::var("DB_HOST").expect("DB_HOST must be set");
    let db_port = env::var("PGPORT").expect("PGPORT must be set");

    let database_url = format!("postgres://{db_user}:{db_password}@{db_host}:{db_port}/{db_name}");
    PgConnection::establish(&database_url)
        .unwrap_or_else(|_| panic!("Error connecting to {database_url}"))
}

pub fn run_db_migrations(conn: &mut impl MigrationHarness<pg::Pg>) {
    info!("Running migrations...");
    let res = conn
        .run_pending_migrations(MIGRATIONS)
        .unwrap_or_else(|error| panic!("Could not run migrations {error}"));
    info!("Migrations completed: {res:?}");
}

pub const DEFAULT_PAGE_SIZE: i64 = 10;
