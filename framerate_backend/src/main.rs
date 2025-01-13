extern crate diesel;
extern crate diesel_migrations;

use actix_cors::Cors;
use actix_web::{middleware::Logger, web::Data, App, HttpServer};
use framerate::{db, movie_entry, routes, show_entry, tmdb, utils};
use std::env;
use tracing::info;
use tracing_log::LogTracer;
use utoipa::{
    openapi::security::{Http, HttpAuthScheme, SecurityScheme},
    Modify, OpenApi,
};
use utoipa_actix_web::{scope, AppExt};
use utoipa_swagger_ui::{Config, SwaggerUi};

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    LogTracer::init().ok();
    utils::log::setup_logger();

    let pool = db::get_connection_pool();
    let mut conn = pool.get().unwrap();
    db::run_db_migrations(&mut conn);

    // Don't use caching for production until appropriate clean-up solution is implemented
    let client = tmdb::get_client(false);

    show_entry::jobs::create_show_entry_metadata_updater(pool.clone(), client.clone());
    movie_entry::jobs::create_movie_entry_metadata_updater(pool.clone(), client.clone());

    let host = env::var("HOST").unwrap_or("localhost".to_string());
    let port = env::var("PORT").unwrap_or("3000".to_string());

    info!("Server starting at http://{host}:{port}");

    struct SecurityAddon;

    impl Modify for SecurityAddon {
        fn modify(&self, openapi: &mut utoipa::openapi::OpenApi) {
            let components = openapi.components.as_mut().unwrap();
            components.add_security_scheme(
                "bearerAuth",
                SecurityScheme::Http(Http::new(HttpAuthScheme::Bearer)),
            )
        }
    }

    #[derive(OpenApi)]
    #[openapi(
        security(("bearerAuth" = [])),
        modifiers(&SecurityAddon),
        info(title = "Framerate API", license(name = "GNU General Public License version 3", identifier = "GPL-3.0-or-later"))
    )]
    struct ApiDoc;

    HttpServer::new(move || {
        App::new()
            .into_utoipa_app()
            .openapi(ApiDoc::openapi())
            .app_data(Data::new(pool.clone()))
            .app_data(Data::new(client.clone()))
            .map(|app| app.wrap(Cors::default()))
            .map(|app| app.wrap(Logger::default()))
            .map(|app| app.wrap(actix_cors::Cors::permissive()))
            .service(scope::scope("/api/v1").configure(routes::init_routes))
            .openapi_service(|api| {
                SwaggerUi::new("/swagger-ui/{_:.*}")
                    .url("/api-docs/openapi.json", api)
                    .config(Config::default().persist_authorization(true))
            })
            .into_app()
            .configure(routes::init_extra_routes)
    })
    .bind(format!("{host}:{port}"))?
    .run()
    .await
}
