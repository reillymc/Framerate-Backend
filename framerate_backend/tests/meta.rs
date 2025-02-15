pub mod common;

mod get_client_config {
    use crate::common::{data, process, setup};
    use actix_http::header::AUTHORIZATION;
    use actix_web::test;
    use framerate::meta::{get_client_config, ClientConfig};
    use serde::Serialize;

    #[derive(Serialize)]
    pub struct SecretBody {
        pub secret: String,
    }

    #[actix_web::test]
    async fn should_require_authentication() {
        let (app, _) = setup::create_app(get_client_config).await;

        let request = test::TestRequest::get()
            .uri("/meta/client_config")
            .to_request();

        let response = test::call_service(&app, request).await;
        assert_eq!(401, response.status());
    }

    #[actix_web::test]
    async fn should_return_default_config_when_unset() {
        let (app, pool) = setup::create_app(get_client_config).await;
        let (token, _) = {
            let mut conn = pool.get().unwrap();
            data::create_authed_user(&mut conn)
        };

        let request = test::TestRequest::get()
            .uri("/meta/client_config")
            .insert_header((AUTHORIZATION, format!("Bearer {token}")))
            .to_request();

        let response = test::call_service(&app, request).await;
        assert!(response.status().is_success());
    }

    #[actix_web::test]
    async fn should_return_config() {
        let (app, pool) = setup::create_app(get_client_config).await;
        let (token, config) = {
            let mut conn: r2d2::PooledConnection<
                diesel::r2d2::ConnectionManager<diesel::PgConnection>,
            > = pool.get().unwrap();
            let config = data::create_client_config(&mut conn);
            let (token, _) = data::create_authed_user(&mut conn);
            (token, config)
        };

        let request = test::TestRequest::get()
            .uri("/meta/client_config")
            .insert_header((AUTHORIZATION, format!("Bearer {token}")))
            .to_request();
        let response = test::call_service(&app, request).await;
        assert!(response.status().is_success());

        let result = process::parse_body::<ClientConfig>(response).await;

        assert_eq!(
            config.media_external_links.len(),
            result.data.media_external_links.len(),
        )
    }
}

mod update_client_config {
    use crate::common::{data, process, setup};
    use actix_http::header::AUTHORIZATION;
    use actix_web::test;
    use framerate::meta::{update_client_config, ClientConfig};
    use serde::Serialize;

    #[derive(Serialize)]
    pub struct SecretBody {
        pub secret: String,
    }

    #[actix_web::test]
    async fn should_require_authentication() {
        let (app, _) = setup::create_app(update_client_config).await;

        let config = data::generate_client_config();

        let request = test::TestRequest::put()
            .uri("/meta/client_config")
            .set_json(config)
            .to_request();

        let response = test::call_service(&app, request).await;
        assert_eq!(401, response.status());
    }

    #[actix_web::test]
    async fn should_require_admin_authentication() {
        let (app, pool) = setup::create_app(update_client_config).await;
        let (token, _) = {
            let mut conn = pool.get().unwrap();
            data::create_authed_user(&mut conn)
        };

        let config = data::generate_client_config();

        let request = test::TestRequest::put()
            .uri("/meta/client_config")
            .set_json(config)
            .insert_header((AUTHORIZATION, format!("Bearer {token}")))
            .to_request();

        let response = test::call_service(&app, request).await;
        assert_eq!(401, response.status());
    }

    #[actix_web::test]
    async fn should_update_client_config() {
        let (app, pool) = setup::create_app(update_client_config).await;
        let (token, _) = {
            let mut conn = pool.get().unwrap();
            data::create_authed_admin_user(&mut conn)
        };

        let config = data::generate_client_config();

        let request = test::TestRequest::put()
            .uri("/meta/client_config")
            .set_json(&config)
            .insert_header((AUTHORIZATION, format!("Bearer {token}")))
            .to_request();
        let response = test::call_service(&app, request).await;
        assert!(response.status().is_success());

        let result = process::parse_body::<ClientConfig>(response).await;
        assert_eq!(
            config.media_external_links.len(),
            result.data.media_external_links.len()
        );
    }
}
