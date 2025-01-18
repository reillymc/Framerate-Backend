pub mod common;

mod generate_setup_token {
    use crate::common::{data, process, setup};
    use actix_web::test;
    use framerate::administration::generate_setup_token;
    use serde::Serialize;
    use std::env;
    use uuid::Uuid;

    #[derive(Serialize)]
    pub struct SecretBody {
        pub secret: String,
    }

    #[actix_web::test]
    async fn should_prevent_setup_when_secret_unset() {
        let (app, pool) = setup::create_app(generate_setup_token).await;
        let _ = {
            let mut conn = pool.get().unwrap();
            data::create_user(&mut conn)
        };

        let secret = Uuid::new_v4().to_string();

        env::remove_var("SETUP_SECRET");

        let request = test::TestRequest::post()
            .uri("/administration/generate_setup_token")
            .set_json(SecretBody { secret })
            .to_request();
        let response = test::call_service(&app, request).await;
        assert_eq!(500, response.status());
    }

    #[actix_web::test]
    async fn should_prevent_setup_when_secret_incorrect() {
        let (app, pool) = setup::create_app(generate_setup_token).await;
        let _ = {
            let mut conn = pool.get().unwrap();
            data::create_user(&mut conn)
        };

        env::set_var("SETUP_SECRET", Uuid::new_v4().to_string());

        let request = test::TestRequest::post()
            .uri("/administration/generate_setup_token")
            .set_json(SecretBody {
                secret: Uuid::new_v4().to_string(),
            })
            .to_request();
        let response = test::call_service(&app, request).await;
        assert_eq!(401, response.status().as_u16());
    }

    #[actix_web::test]
    async fn should_prevent_setup_when_db_has_users() {
        let (app, pool) = setup::create_app(generate_setup_token).await;
        let _ = {
            let mut conn = pool.get().unwrap();
            data::create_user(&mut conn)
        };

        let secret = Uuid::new_v4().to_string();

        env::set_var("SETUP_SECRET", secret.clone());

        let request = test::TestRequest::post()
            .uri("/administration/generate_setup_token")
            .set_json(SecretBody { secret })
            .to_request();
        let response = test::call_service(&app, request).await;
        assert_eq!(401, response.status().as_u16());
    }

    #[actix_web::test]
    async fn should_return_setup_token() {
        let (app, _) = setup::create_app(generate_setup_token).await;

        let secret = Uuid::new_v4().to_string();

        env::set_var("SETUP_SECRET", secret.clone());

        let request = test::TestRequest::post()
            .uri("/administration/generate_setup_token")
            .set_json(SecretBody { secret })
            .to_request();
        let response = test::call_service(&app, request).await;
        assert!(response.status().is_success());

        let body = process::parse_body::<String>(response).await;
        assert!(body.data.len() > 0)
    }
}

mod generate_invite {
    use crate::common::{data, process, setup};
    use actix_http::header::AUTHORIZATION;
    use actix_web::test;
    use framerate::{
        administration::{generate_invite, InviteDetails},
        utils::invite::decode_invite,
    };
    use serde::Serialize;
    use std::env;
    use uuid::Uuid;

    #[derive(Serialize)]
    pub struct SecretBody {
        pub secret: String,
    }

    #[actix_web::test]
    async fn should_require_authentication() {
        let (app, _) = setup::create_app(generate_invite).await;

        env::set_var("REGISTRATION_MODE", "invite");

        let request = test::TestRequest::post()
            .uri("/administration/generate_invite")
            .set_json(InviteDetails {
                email: Uuid::new_v4().to_string(),
            })
            .to_request();

        let response = test::call_service(&app, request).await;
        assert_eq!(401, response.status());
    }

    #[actix_web::test]
    async fn should_require_admin_authentication() {
        let (app, pool) = setup::create_app(generate_invite).await;
        let (token, _) = {
            let mut conn = pool.get().unwrap();
            data::create_authed_user(&mut conn)
        };

        env::set_var("REGISTRATION_MODE", "invite");

        let request = test::TestRequest::post()
            .uri("/administration/generate_invite")
            .set_json(InviteDetails {
                email: Uuid::new_v4().to_string(),
            })
            .insert_header((AUTHORIZATION, format!("Bearer {token}")))
            .to_request();

        let response = test::call_service(&app, request).await;
        assert_eq!(401, response.status());
    }

    #[actix_web::test]
    async fn should_prevent_invite_generation_when_secret_unset_or_unknown() {
        let (app, pool) = setup::create_app(generate_invite).await;
        let (token, _) = {
            let mut conn = pool.get().unwrap();
            data::create_authed_admin_user(&mut conn)
        };

        env::remove_var("REGISTRATION_MODE");

        let request = test::TestRequest::post()
            .uri("/administration/generate_invite")
            .set_json(InviteDetails {
                email: Uuid::new_v4().to_string(),
            })
            .insert_header((AUTHORIZATION, format!("Bearer {token}")))
            .to_request();
        let response = test::call_service(&app, request).await;
        assert_eq!(401, response.status());

        env::set_var("REGISTRATION_MODE", Uuid::new_v4().to_string());

        let request = test::TestRequest::post()
            .uri("/administration/generate_invite")
            .set_json(InviteDetails {
                email: Uuid::new_v4().to_string(),
            })
            .insert_header((AUTHORIZATION, format!("Bearer {token}")))
            .to_request();
        let response = test::call_service(&app, request).await;
        assert_eq!(500, response.status());
    }

    #[actix_web::test]
    async fn should_require_email() {
        let (app, pool) = setup::create_app(generate_invite).await;
        let (token, _) = {
            let mut conn = pool.get().unwrap();
            data::create_authed_admin_user(&mut conn)
        };

        env::set_var("REGISTRATION_MODE", "invite");

        let request = test::TestRequest::post()
            .uri("/administration/generate_invite")
            .insert_header((AUTHORIZATION, format!("Bearer {token}")))
            .to_request();
        let response = test::call_service(&app, request).await;
        assert_eq!(400, response.status());

        let request = test::TestRequest::post()
            .uri("/administration/generate_invite")
            .set_json(InviteDetails {
                email: "".to_string(),
            })
            .insert_header((AUTHORIZATION, format!("Bearer {token}")))
            .to_request();
        let response = test::call_service(&app, request).await;
        assert_eq!(400, response.status());
    }

    #[actix_web::test]
    async fn should_generate_invite() {
        let (app, pool) = setup::create_app(generate_invite).await;
        let (token, _) = {
            let mut conn = pool.get().unwrap();
            data::create_authed_admin_user(&mut conn)
        };

        env::set_var("REGISTRATION_MODE", "invite");

        let invite_details = InviteDetails {
            email: Uuid::new_v4().to_string(),
        };

        let request = test::TestRequest::post()
            .uri("/administration/generate_invite")
            .set_json(&invite_details)
            .insert_header((AUTHORIZATION, format!("Bearer {token}")))
            .to_request();
        let response = test::call_service(&app, request).await;
        assert!(response.status().is_success());

        let body = process::parse_body::<String>(response).await;
        assert!(body.data.len() > 0);

        let decoded_invite = decode_invite(&body.data).unwrap();

        assert_eq!(invite_details.email, decoded_invite.email);
    }
}
