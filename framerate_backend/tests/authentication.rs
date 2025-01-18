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
        pub email: String,
        pub password: String,
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

        let request = test::TestRequest::post()
            .uri("/authentication/login")
            .to_request();
        let response = test::call_service(&app, request).await;
        assert_eq!(400, response.status().as_u16());

        let request = test::TestRequest::post()
            .uri("/authentication/login")
            .set_json(LoginBody {
                email: "".to_string(),
                password: "".to_string(),
            })
            .to_request();
        let response = test::call_service(&app, request).await;
        assert_eq!(400, response.status().as_u16());

        let request = test::TestRequest::post()
            .uri("/authentication/login")
            .set_json(LoginBody {
                email: user.email,
                password: "".to_string(),
            })
            .to_request();
        let response = test::call_service(&app, request).await;
        assert_eq!(400, response.status().as_u16());

        let request = test::TestRequest::post()
            .uri("/authentication/login")
            .set_json(LoginBody {
                email: "".to_string(),
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

        user.password = Uuid::new_v4().to_string();

        let request = test::TestRequest::post()
            .uri("/authentication/login")
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
                email: Uuid::new_v4().to_string(),
                password: Uuid::new_v4().to_string(),
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
                email: Uuid::new_v4().to_string(),
                password: Uuid::new_v4().to_string(),
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
                    registered_user_save
                        .clone()
                        .password(User::hash_password(&registered_user_save.password).unwrap()),
                    non_authenticatable_user_save.clone().password(
                        User::hash_password(&non_authenticatable_user_save.password).unwrap(),
                    ),
                ])
                .execute(&mut conn)
                .ok();

            (registered_user, non_authenticatable_user)
        };

        let request = test::TestRequest::post()
            .uri("/authentication/login")
            .set_json(LoginBody::from(&registered_user))
            .to_request();

        let response = test::call_service(&app, request).await;
        assert_eq!(401, response.status().as_u16());

        let request = test::TestRequest::post()
            .uri("/authentication/login")
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
            .uri("/authentication/login")
            .set_json(user)
            .to_request();

        let response = test::call_service(&app, request).await;
        assert!(response.status().is_success());
    }
}

mod register {
    use crate::common::{data, process, setup};
    use actix_web::test;
    use framerate::{
        authentication::{register, LoginResponse},
        utils::invite::create_invite,
    };
    use std::env;
    use uuid::Uuid;

    #[actix_web::test]
    async fn should_prevent_registration_when_secret_unset_or_unknown() {
        let (app, _) = setup::create_app(register).await;

        env::remove_var("REGISTRATION_MODE");

        let request = test::TestRequest::post()
            .uri("/authentication/register")
            .set_json(data::generate_save_registering_user())
            .to_request();
        let response = test::call_service(&app, request).await;
        assert_eq!(401, response.status());

        env::set_var("REGISTRATION_MODE", Uuid::new_v4().to_string());

        let request = test::TestRequest::post()
            .uri("/authentication/register")
            .set_json(data::generate_save_registering_user())
            .to_request();
        let response = test::call_service(&app, request).await;
        assert_eq!(401, response.status());
    }

    #[actix_web::test]
    async fn should_prevent_invite_registration_with_no_invite() {
        let (app, _) = setup::create_app(register).await;

        env::set_var("REGISTRATION_MODE", "invite");

        let registering_user = data::generate_save_registering_user();

        let request = test::TestRequest::post()
            .uri("/authentication/register")
            .set_json(&registering_user)
            .to_request();
        let response = test::call_service(&app, request).await;
        assert_eq!(401, response.status().as_u16());
    }

    #[actix_web::test]
    async fn should_prevent_invite_registration_with_invalid_invite() {
        let (app, _) = setup::create_app(register).await;

        env::set_var("REGISTRATION_MODE", "invite");

        let registering_user = data::generate_save_registering_user()
            .invite_code(create_invite(Uuid::new_v4().to_string()).unwrap());

        let request = test::TestRequest::post()
            .uri("/authentication/register")
            .set_json(&registering_user)
            .to_request();
        let response = test::call_service(&app, request).await;
        assert_eq!(401, response.status().as_u16());
    }

    #[actix_web::test]
    async fn should_register_user_with_valid_invite() {
        let (app, _) = setup::create_app(register).await;

        env::set_var("REGISTRATION_MODE", "invite");

        let registering_user = data::generate_save_registering_user();

        let invite_code = create_invite(registering_user.email.clone()).unwrap();

        let registering_user = registering_user.invite_code(invite_code);

        let request = test::TestRequest::post()
            .uri("/authentication/register")
            .set_json(&registering_user)
            .to_request();
        let response = test::call_service(&app, request).await;
        assert!(response.status().is_success());
        let body = process::parse_body::<LoginResponse>(response).await;
        assert!(body.data.token.len() > 0)
    }

    #[actix_web::test]
    async fn should_register_user_when_mode_open() {
        let (app, _) = setup::create_app(register).await;

        env::set_var("REGISTRATION_MODE", "open");

        let registering_user = data::generate_save_registering_user();

        let request = test::TestRequest::post()
            .uri("/authentication/register")
            .set_json(&registering_user)
            .to_request();
        let response = test::call_service(&app, request).await;
        assert!(response.status().is_success());
        let body = process::parse_body::<LoginResponse>(response).await;
        assert!(body.data.token.len() > 0)
    }
}
