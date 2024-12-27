pub mod common;

mod login {
    use crate::common::{data, setup};
    use actix_web::test;
    use diesel::RunQueryDsl;
    use framerate::{
        authentication::login,
        schema::users,
        user::{PermissionLevel, User},
    };
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

    impl From<&User> for LoginBody {
        fn from(user: &User) -> Self {
            LoginBody {
                email: user.email.clone(),
                password: user.password.clone(),
            }
        }
    }

    #[actix_web::test]
    async fn should_require_email_and_password() {
        let (app, pool) = setup::create_app(login).await;
        let user = {
            let mut conn = pool.get().unwrap();
            data::create_user(&mut conn)
        };

        let request = test::TestRequest::post().uri("/auth/login").to_request();
        let response = test::call_service(&app, request).await;
        assert_eq!(400, response.status().as_u16());

        let request = test::TestRequest::post()
            .uri("/auth/login")
            .set_json(LoginBody {
                email: None,
                password: None,
            })
            .to_request();
        let response = test::call_service(&app, request).await;
        assert_eq!(400, response.status().as_u16());

        let request = test::TestRequest::post()
            .uri("/auth/login")
            .set_json(LoginBody {
                email: user.email.clone(),
                password: None,
            })
            .to_request();
        let response = test::call_service(&app, request).await;
        assert_eq!(400, response.status().as_u16());

        let request = test::TestRequest::post()
            .uri("/auth/login")
            .set_json(LoginBody {
                email: None,
                password: user.password.clone(),
            })
            .to_request();
        let response = test::call_service(&app, request).await;
        assert_eq!(400, response.status().as_u16());

        let request = test::TestRequest::post()
            .uri("/auth/login")
            .set_json(LoginBody {
                email: Some("".to_string()),
                password: Some("".to_string()),
            })
            .to_request();
        let response = test::call_service(&app, request).await;
        assert_eq!(400, response.status().as_u16());

        let request = test::TestRequest::post()
            .uri("/auth/login")
            .set_json(LoginBody {
                email: user.email,
                password: Some("".to_string()),
            })
            .to_request();
        let response = test::call_service(&app, request).await;
        assert_eq!(400, response.status().as_u16());

        let request = test::TestRequest::post()
            .uri("/auth/login")
            .set_json(LoginBody {
                email: Some("".to_string()),
                password: user.password,
            })
            .to_request();
        let response = test::call_service(&app, request).await;
        assert_eq!(400, response.status().as_u16());
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
        assert_eq!(401, response.status().as_u16());
    }

    #[actix_web::test]
    async fn should_not_authenticate_non_authenticatable_user() {
        let (app, pool) = setup::create_app(login).await;
        let (registered_user, non_authenticatable_user) = {
            let mut conn = pool.get().unwrap();
            let registered_user = User {
                user_id: Uuid::new_v4(),
                email: Some(Uuid::new_v4().to_string()),
                password: Some(Uuid::new_v4().to_string()),
                first_name: Uuid::new_v4().to_string(),
                last_name: Uuid::new_v4().to_string(),
                date_created: chrono::Local::now().naive_local(),
                permission_level: i16::from(PermissionLevel::Registered),
                public: false,
                avatar_uri: Some(Uuid::new_v4().to_string()),
                configuration: serde_json::json!({
                    "people": [],
                    "venues": [],
                }),
                created_by: None,
            };

            let non_authenticatable_user = User {
                user_id: Uuid::new_v4(),
                email: Some(Uuid::new_v4().to_string()),
                password: Some(Uuid::new_v4().to_string()),
                first_name: Uuid::new_v4().to_string(),
                last_name: Uuid::new_v4().to_string(),
                date_created: chrono::Local::now().naive_local(),
                permission_level: i16::from(PermissionLevel::NonAuthenticatable),
                public: false,
                avatar_uri: Some(Uuid::new_v4().to_string()),
                configuration: serde_json::json!({
                    "people": [],
                    "venues": [],
                }),
                created_by: None,
            };

            let registered_user_save = registered_user.clone();
            let non_authenticatable_user_save = non_authenticatable_user.clone();

            diesel::insert_into(users::table)
                .values(vec![
                    registered_user_save.clone().password(
                        User::hash_password(&registered_user_save.password.unwrap()).unwrap(),
                    ),
                    non_authenticatable_user_save.clone().password(
                        User::hash_password(&non_authenticatable_user_save.password.unwrap())
                            .unwrap(),
                    ),
                ])
                .execute(&mut conn)
                .ok();

            (registered_user, non_authenticatable_user)
        };

        let request = test::TestRequest::post()
            .uri("/auth/login")
            .set_json(LoginBody::from(&registered_user))
            .to_request();

        let response = test::call_service(&app, request).await;
        assert_eq!(401, response.status().as_u16());

        let request = test::TestRequest::post()
            .uri("/auth/login")
            .set_json(LoginBody::from(&non_authenticatable_user))
            .to_request();

        let response = test::call_service(&app, request).await;
        assert_eq!(401, response.status().as_u16());
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
        assert!(response.status().is_server_error());
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
        assert_eq!(401, response.status().as_u16());
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
        assert_eq!(401, response.status().as_u16());
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
