pub mod common;

mod user_common {
    use chrono::NaiveDateTime;
    use serde::Deserialize;
    use uuid::Uuid;

    #[derive(Deserialize)]
    #[serde(rename_all = "camelCase")]
    pub struct TestUserResponse {
        pub user_id: Uuid,
        pub email: Option<String>,
        pub first_name: String,
        pub last_name: String,
        pub avatar_uri: Option<String>,
        pub configuration: Option<serde_json::Value>,
        // Added field to enable testing that password is never returned
        pub password: Option<String>,
        pub permission_level: Option<i16>,
        pub public: Option<bool>,
        pub date_created: Option<NaiveDateTime>,
        pub created_by: Option<Uuid>,
    }
}

mod find_all {
    use crate::{
        common::{data, process, setup},
        user_common::TestUserResponse,
    };
    use actix_web::{http::header::AUTHORIZATION, test};
    use framerate::user::find_all;

    #[actix_web::test]
    async fn should_require_authentication() {
        let (app, _) = setup::create_app(find_all).await;

        let request = test::TestRequest::get().uri("/users").to_request();

        let response = test::call_service(&app, request).await;
        assert_eq!(401, response.status());
    }

    #[actix_web::test]
    async fn should_require_admin_authentication() {
        let (app, pool) = setup::create_app(find_all).await;

        let token = {
            let mut conn = pool.get().unwrap();
            let (token, _) = data::create_authed_user(&mut conn);
            token
        };

        let request = test::TestRequest::get()
            .uri("/users")
            .insert_header((AUTHORIZATION, format!("Bearer {token}")))
            .to_request();

        let response = test::call_service(&app, request).await;
        assert_eq!(401, response.status());
    }

    #[actix_web::test]
    async fn should_return_users() {
        let (app, pool) = setup::create_app(find_all).await;

        let (current_user, other_user, token) = {
            let mut conn = pool.get().unwrap();
            let (token, current_user) = data::create_authed_admin_user(&mut conn);
            let other_user = data::create_user(&mut conn);
            (current_user, other_user, token)
        };

        let request = test::TestRequest::get()
            .uri("/users")
            .insert_header((AUTHORIZATION, format!("Bearer {token}")))
            .to_request();

        let response = test::call_service(&app, request).await;
        assert!(response.status().is_success());

        let mut result = process::parse_body::<Vec<TestUserResponse>>(response).await;
        assert_eq!(2, result.data.len());

        let user1 = result.data.pop().unwrap();
        let user2 = result.data.pop().unwrap();

        assert_eq!(None, user1.email);
        assert_eq!(None, user1.password);
        assert_eq!(None, user1.configuration);

        assert_eq!(None, user2.email);
        assert_eq!(None, user2.password);
        assert_eq!(None, user2.configuration);

        if current_user.user_id == user1.user_id {
            assert_eq!(current_user.avatar_uri, user1.avatar_uri);
            assert_eq!(current_user.first_name, user1.first_name);
            assert_eq!(current_user.last_name, user1.last_name);
            assert_eq!(current_user.avatar_uri, user1.avatar_uri);

            assert_eq!(other_user.avatar_uri, user2.avatar_uri);
            assert_eq!(other_user.first_name, user2.first_name);
            assert_eq!(other_user.last_name, user2.last_name);
            assert_eq!(other_user.avatar_uri, user2.avatar_uri);
        } else {
            assert_eq!(other_user.avatar_uri, user1.avatar_uri);
            assert_eq!(other_user.first_name, user1.first_name);
            assert_eq!(other_user.last_name, user1.last_name);
            assert_eq!(other_user.avatar_uri, user1.avatar_uri);

            assert_eq!(current_user.avatar_uri, user2.avatar_uri);
            assert_eq!(current_user.first_name, user2.first_name);
            assert_eq!(current_user.last_name, user2.last_name);
            assert_eq!(current_user.avatar_uri, user2.avatar_uri);
        }
    }
}

mod find {
    use actix_http::header::AUTHORIZATION;
    use actix_web::test;
    use framerate::user::find;

    use crate::{
        common::{data, process, setup},
        user_common::TestUserResponse,
    };

    #[actix_web::test]
    async fn should_require_authentication() {
        let (app, pool) = setup::create_app(find).await;

        let user = {
            let mut conn = pool.get().unwrap();
            data::create_user(&mut conn)
        };

        let request = test::TestRequest::get()
            .uri(&format!("/users/{}", user.user_id))
            .to_request();

        let response = test::call_service(&app, request).await;
        assert_eq!(401, response.status());
    }

    #[actix_web::test]
    async fn should_return_scoped_other_user_details() {
        let (app, pool) = setup::create_app(find).await;

        let (token, other_user) = {
            let mut conn = pool.get().unwrap();
            let other_user = data::create_user(&mut conn);
            let (token, _) = data::create_authed_user(&mut conn);

            (token, other_user)
        };

        let request = test::TestRequest::get()
            .uri(&format!("/users/{}", other_user.user_id))
            .insert_header((AUTHORIZATION, format!("Bearer {token}")))
            .to_request();

        let response = test::call_service(&app, request).await;
        assert!(response.status().is_success());

        let result = process::parse_body::<TestUserResponse>(response).await;

        assert_eq!(None, result.data.email);
        assert_eq!(None, result.data.password);
        assert_eq!(None, result.data.configuration);
        assert_eq!(other_user.user_id, result.data.user_id);
        assert_eq!(other_user.avatar_uri, result.data.avatar_uri);
        assert_eq!(other_user.first_name, result.data.first_name);
        assert_eq!(other_user.last_name, result.data.last_name);
    }

    #[actix_web::test]
    async fn should_return_full_current_user_details() {
        let (app, pool) = setup::create_app(find).await;

        let (token, current_user) = {
            let mut conn = pool.get().unwrap();
            data::create_authed_user(&mut conn)
        };

        let request = test::TestRequest::get()
            .uri(&format!("/users/{}", current_user.user_id))
            .insert_header((AUTHORIZATION, format!("Bearer {token}")))
            .to_request();

        let response = test::call_service(&app, request).await;
        assert!(response.status().is_success());

        let result = process::parse_body::<TestUserResponse>(response).await;

        assert_eq!(None, result.data.password);

        assert_eq!(current_user.email, result.data.email.unwrap());
        assert_eq!(Some(current_user.configuration), result.data.configuration);
        assert_eq!(current_user.user_id, result.data.user_id);
        assert_eq!(current_user.avatar_uri, result.data.avatar_uri);
        assert_eq!(current_user.first_name, result.data.first_name);
        assert_eq!(current_user.last_name, result.data.last_name);
    }
}

mod create {
    use crate::{
        common::{data, process, setup},
        user_common::TestUserResponse,
    };
    use actix_web::{http::header::AUTHORIZATION, test};
    use framerate::{user::create, user::PermissionLevel};

    #[actix_web::test]
    async fn should_require_authentication() {
        let (app, _) = setup::create_app(create).await;

        let user = data::generate_save_new_user();

        let request = test::TestRequest::post()
            .uri("/users")
            .set_json(user)
            .to_request();

        let response = test::call_service(&app, request).await;
        assert_eq!(401, response.status());
    }

    #[actix_web::test]
    async fn should_require_admin_privileges() {
        let (app, pool) = setup::create_app(create).await;

        let (token, _) = {
            let mut conn = pool.get().unwrap();
            data::create_authed_user(&mut conn)
        };

        let user = data::generate_save_new_user();

        let request = test::TestRequest::post()
            .uri("/users")
            .insert_header((AUTHORIZATION, format!("Bearer {token}")))
            .set_json(user)
            .to_request();

        let response = test::call_service(&app, request).await;
        assert_eq!(401, response.status());
    }

    #[actix_web::test]
    async fn should_create_user() {
        let (app, pool) = setup::create_app(create).await;

        let (token, _) = {
            let mut conn = pool.get().unwrap();
            data::create_authed_admin_user(&mut conn)
        };

        let user = data::generate_save_new_user();

        let request = test::TestRequest::post()
            .uri("/users")
            .insert_header((AUTHORIZATION, format!("Bearer {token}")))
            .set_json(&user)
            .to_request();

        let response = test::call_service(&app, request).await;
        assert!(response.status().is_success());

        let result = process::parse_body::<TestUserResponse>(response).await;

        assert_eq!(None, result.data.password);
        assert_eq!(None, result.data.created_by);

        assert_eq!(user.email, result.data.email.unwrap());
        assert_eq!(user.avatar_uri, result.data.avatar_uri);
        assert_eq!(user.first_name, result.data.first_name);
        assert_eq!(user.last_name, result.data.last_name);
        assert_eq!(
            PermissionLevel::GeneralUser,
            result.data.permission_level.unwrap().into()
        );
    }
}

mod update {
    use crate::{
        common::{data, process, setup},
        user_common::TestUserResponse,
    };
    use actix_web::{http::header::AUTHORIZATION, test};
    use framerate::{user::update, user::PermissionLevel};
    use uuid::Uuid;

    #[actix_web::test]
    async fn should_require_authentication() {
        let (app, pool) = setup::create_app(update).await;

        let (_, user) = {
            let mut conn = pool.get().unwrap();
            data::create_authed_user(&mut conn)
        };

        let request = test::TestRequest::put()
            .uri(&format!("/users/{}", user.user_id))
            .set_json(user)
            .to_request();

        let response = test::call_service(&app, request).await;
        assert_eq!(401, response.status());
    }

    #[actix_web::test]
    async fn should_not_update_other_user() {
        let (app, pool) = setup::create_app(update).await;

        let (token, other_user) = {
            let mut conn = pool.get().unwrap();
            let (token, _) = data::create_authed_user(&mut conn);
            let other_user = data::create_user(&mut conn);
            (token, other_user)
        };

        let request = test::TestRequest::put()
            .uri(&format!("/users/{}", other_user.user_id))
            .insert_header((AUTHORIZATION, format!("Bearer {token}")))
            .set_json(other_user)
            .to_request();

        let response = test::call_service(&app, request).await;
        assert_eq!(404, response.status());
    }

    #[actix_web::test]
    async fn should_update_user() {
        let (app, pool) = setup::create_app(update).await;

        let (token, mut user) = {
            let mut conn = pool.get().unwrap();
            data::create_authed_user(&mut conn)
        };

        user.first_name = Uuid::new_v4().to_string();
        user.last_name = Uuid::new_v4().to_string();
        user.configuration = serde_json::json!({
            "people": [Uuid::new_v4().to_string()],
            "venues": [Uuid::new_v4().to_string()],
        });

        let request = test::TestRequest::put()
            .uri(&format!("/users/{}", user.user_id))
            .insert_header((AUTHORIZATION, format!("Bearer {token}")))
            .set_json(&user)
            .to_request();

        let response = test::call_service(&app, request).await;
        assert!(response.status().is_success());

        let result = process::parse_body::<TestUserResponse>(response).await;

        assert_eq!(None, result.data.password);
        assert_eq!(None, result.data.created_by);

        assert_eq!(user.email, result.data.email.unwrap());
        assert_eq!(user.avatar_uri, result.data.avatar_uri);
        assert_eq!(user.first_name, result.data.first_name);
        assert_eq!(user.last_name, result.data.last_name);
        assert_eq!(user.configuration, result.data.configuration.unwrap());
        assert_eq!(user.date_created, result.data.date_created.unwrap());
        assert_eq!(user.public, result.data.public.unwrap());
        assert_eq!(
            PermissionLevel::GeneralUser,
            result.data.permission_level.unwrap().into()
        );
    }
}
