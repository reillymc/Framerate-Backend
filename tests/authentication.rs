pub mod common;

mod login {
    use crate::common::{data, setup};
    use actix_web::test;
    use framerate::{authentication::login, user::User};
    use serde::Serialize;
    use uuid::Uuid;

    #[derive(Serialize)]
    pub struct LoginBody {
        #[serde(skip_serializing_if = "Option::is_none")]
        pub email: Option<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        pub password: Option<String>,
    }

    impl From<User> for LoginBody {
        fn from(user: User) -> Self {
            LoginBody {
                email: user.email,
                password: user.password,
            }
        }
    }

    #[actix_web::test]
    async fn should_require_email_and_password() {
        let (app, pool) = setup::create_app(login).await;
        let _ = {
            let mut conn = pool.get().unwrap();
            data::create_user(&mut conn)
        };

        let request = test::TestRequest::post().uri("/auth/login").to_request();
        let response = test::call_service(&app, request).await;
        assert_eq!(response.status().as_u16(), 400);

        let request = test::TestRequest::post()
            .uri("/auth/login")
            .set_json(LoginBody {
                email: Some(Uuid::new_v4().to_string()),
                password: None,
            })
            .to_request();
        let response = test::call_service(&app, request).await;
        assert_eq!(response.status().as_u16(), 400);

        let request = test::TestRequest::post()
            .uri("/auth/login")
            .set_json(LoginBody {
                email: None,
                password: Some(Uuid::new_v4().to_string()),
            })
            .to_request();
        let response = test::call_service(&app, request).await;
        assert_eq!(response.status().as_u16(), 400);
    }

    #[actix_web::test]
    async fn should_not_authenticate_invalid_credentials() {
        let (app, pool) = setup::create_app(login).await;
        let mut user = {
            let mut conn = pool.get().unwrap();
            data::create_user(&mut conn)
        };

        user.password = Some(Uuid::new_v4().to_string());

        let request = test::TestRequest::post()
            .uri("/auth/login")
            .set_json(LoginBody::from(user))
            .to_request();

        let response = test::call_service(&app, request).await;
        assert_eq!(response.status().as_u16(), 401);
    }

    #[actix_web::test]
    async fn should_authenticate_valid_credentials() {
        let (app, pool) = setup::create_app(login).await;
        let (_, user) = {
            let mut conn = pool.get().unwrap();
            data::create_authed_user(&mut conn)
        };

        let user = LoginBody::from(user);

        let request = test::TestRequest::post()
            .uri("/auth/login")
            .set_json(user)
            .to_request();

        let response = test::call_service(&app, request).await;
        assert!(response.status().is_success());
    }
}

mod setup {
    use std::env;

    use crate::common::{data, process, setup};
    use actix_web::test;
    use framerate::authentication::setup as setupRoute;
    use serde::Serialize;
    use uuid::Uuid;

    #[derive(Serialize)]
    pub struct SecretBody {
        pub secret: String,
    }

    #[actix_web::test]
    async fn should_prevent_setup_when_secret_unset() {
        let (app, pool) = setup::create_app(setupRoute).await;
        let _ = {
            let mut conn = pool.get().unwrap();
            data::create_user(&mut conn)
        };

        let secret = Uuid::new_v4().to_string();

        env::remove_var("SETUP_SECRET");

        let request = test::TestRequest::post()
            .uri("/auth/setup")
            .set_json(SecretBody { secret })
            .to_request();
        let response = test::call_service(&app, request).await;
        assert_eq!(response.status().as_u16(), 500);
    }

    #[actix_web::test]
    async fn should_prevent_setup_when_secret_incorrect() {
        let (app, pool) = setup::create_app(setupRoute).await;
        let _ = {
            let mut conn = pool.get().unwrap();
            data::create_user(&mut conn)
        };

        env::set_var("SETUP_SECRET", Uuid::new_v4().to_string());

        let request = test::TestRequest::post()
            .uri("/auth/setup")
            .set_json(SecretBody {
                secret: Uuid::new_v4().to_string(),
            })
            .to_request();
        let response = test::call_service(&app, request).await;
        assert_eq!(response.status().as_u16(), 401);
    }

    #[actix_web::test]
    async fn should_prevent_setup_when_db_has_users() {
        let (app, pool) = setup::create_app(setupRoute).await;
        let _ = {
            let mut conn = pool.get().unwrap();
            data::create_user(&mut conn)
        };

        let secret = Uuid::new_v4().to_string();

        env::set_var("SETUP_SECRET", secret.clone());

        let request = test::TestRequest::post()
            .uri("/auth/setup")
            .set_json(SecretBody { secret })
            .to_request();
        let response = test::call_service(&app, request).await;
        assert_eq!(response.status().as_u16(), 401);
    }

    #[actix_web::test]
    async fn should_return_setup_token() {
        let (app, _) = setup::create_app(setupRoute).await;

        let secret = Uuid::new_v4().to_string();

        env::set_var("SETUP_SECRET", secret.clone());

        let request = test::TestRequest::post()
            .uri("/auth/setup")
            .set_json(SecretBody { secret })
            .to_request();
        let response = test::call_service(&app, request).await;
        assert!(response.status().is_success());

        let body = process::parse_body::<String>(response).await;
        assert!(body.data.len() > 0)
    }
}
