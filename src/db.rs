use diesel::pg;
use diesel::prelude::*;
use diesel_migrations::{embed_migrations, EmbeddedMigrations, MigrationHarness};
use dotenvy::dotenv;
use std::env;

const MIGRATIONS: EmbeddedMigrations = embed_migrations!("migrations");
pub fn establish_connection() -> PgConnection {
    dotenv().ok();

    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    PgConnection::establish(&database_url)
        .unwrap_or_else(|_| panic!("Error connecting to {}", database_url))
}

pub fn run_db_migrations(conn: &mut impl MigrationHarness<pg::Pg>) {
    conn.run_pending_migrations(MIGRATIONS)
        .expect("Could not run migrations");
}
