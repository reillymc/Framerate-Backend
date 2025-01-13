pub mod common;

mod find_all {
    use crate::common::{data, process, setup};
    use actix_web::{http::header::AUTHORIZATION, test};
    use framerate::company::{find_all, Company};

    #[actix_web::test]
    async fn should_require_authentication() {
        let (app, _) = setup::create_app(find_all).await;

        let request = test::TestRequest::get().uri("/company").to_request();

        let response = test::call_service(&app, request).await;
        assert_eq!(401, response.status());
    }

    #[actix_web::test]
    async fn should_not_return_other_users_company() {
        let (app, pool) = setup::create_app(find_all).await;

        let token = {
            let mut conn = pool.get().unwrap();
            let user = data::create_user(&mut conn);
            data::create_company(&mut conn, &user);
            let (token, _) = data::create_authed_user(&mut conn);
            token
        };

        let request = test::TestRequest::get()
            .uri("/company")
            .insert_header((AUTHORIZATION, format!("Bearer {token}")))
            .to_request();

        let response = test::call_service(&app, request).await;
        assert!(response.status().is_success());

        let result = process::parse_body::<Vec<Company>>(response).await;
        assert_eq!(0, result.data.len());
    }

    #[actix_web::test]
    async fn should_return_users() {
        let (app, pool) = setup::create_app(find_all).await;

        let (token, user, company) = {
            let mut conn = pool.get().unwrap();
            let (token, user) = data::create_authed_user(&mut conn);
            let company = data::create_company(&mut conn, &user);
            (token, user, company)
        };

        let request = test::TestRequest::get()
            .uri("/company")
            .insert_header((AUTHORIZATION, format!("Bearer {token}")))
            .to_request();

        let response = test::call_service(&app, request).await;
        assert!(response.status().is_success());

        let result = process::parse_body::<Vec<Company>>(response).await;
        assert_eq!(1, result.data.len());

        let company_response = result.data.first().unwrap();

        assert_eq!(user.user_id, company_response.created_by);
        assert_eq!(company.first_name, company_response.first_name);
        assert_eq!(company.last_name, company_response.last_name);
    }
}

mod create {
    use crate::common::{data, process, setup};
    use actix_web::{http::header::AUTHORIZATION, test};
    use framerate::company::{create, Company};

    #[actix_web::test]
    async fn should_require_authentication() {
        let (app, _) = setup::create_app(create).await;

        let company = data::generate_save_company();

        let request = test::TestRequest::post()
            .uri("/company")
            .set_json(company)
            .to_request();

        let response = test::call_service(&app, request).await;
        assert_eq!(401, response.status());
    }

    #[actix_web::test]
    async fn should_create_company() {
        let (app, pool) = setup::create_app(create).await;

        let (token, user) = {
            let mut conn = pool.get().unwrap();
            data::create_authed_user(&mut conn)
        };

        let company = data::generate_save_company();

        let request = test::TestRequest::post()
            .uri("/company")
            .insert_header((AUTHORIZATION, format!("Bearer {token}")))
            .set_json(&company)
            .to_request();

        let response = test::call_service(&app, request).await;
        assert!(response.status().is_success());

        let result = process::parse_body::<Company>(response).await;

        assert_eq!(user.user_id, result.data.created_by);
        assert_eq!(company.first_name, result.data.first_name);
        assert_eq!(company.last_name, result.data.last_name);
    }
}

mod update {
    use crate::common::{data, process, setup};
    use actix_web::{http::header::AUTHORIZATION, test};
    use framerate::company::{update, Company};
    use uuid::Uuid;

    #[actix_web::test]
    async fn should_require_authentication() {
        let (app, pool) = setup::create_app(update).await;

        let company = {
            let mut conn = pool.get().unwrap();
            let (_, user) = data::create_authed_user(&mut conn);
            data::create_company(&mut conn, &user)
        };

        let request = test::TestRequest::put()
            .uri(&format!("/company/{}", company.company_id))
            .set_json(company)
            .to_request();

        let response = test::call_service(&app, request).await;
        assert_eq!(401, response.status());
    }

    #[actix_web::test]
    async fn should_not_update_other_users_company() {
        let (app, pool) = setup::create_app(update).await;

        let (token, company) = {
            let mut conn = pool.get().unwrap();
            let (token, _) = data::create_authed_user(&mut conn);
            let other_user = data::create_user(&mut conn);
            let company = data::create_company(&mut conn, &other_user);

            (token, company)
        };

        let request = test::TestRequest::put()
            .uri(&format!("/company/{}", company.company_id))
            .insert_header((AUTHORIZATION, format!("Bearer {token}")))
            .set_json(company)
            .to_request();

        let response = test::call_service(&app, request).await;
        assert_eq!(404, response.status());
    }

    #[actix_web::test]
    async fn should_update_company() {
        let (app, pool) = setup::create_app(update).await;

        let (token, user, mut company) = {
            let mut conn = pool.get().unwrap();
            let (token, user) = data::create_authed_user(&mut conn);
            let company = data::create_company(&mut conn, &user);

            (token, user, company)
        };

        company.first_name = Uuid::new_v4().to_string();
        company.last_name = Uuid::new_v4().to_string();

        let request = test::TestRequest::put()
            .uri(&format!("/company/{}", company.company_id))
            .insert_header((AUTHORIZATION, format!("Bearer {token}")))
            .set_json(&company)
            .to_request();

        let response = test::call_service(&app, request).await;
        assert!(response.status().is_success());

        let result = process::parse_body::<Company>(response).await;

        assert_eq!(user.user_id, result.data.created_by);
        assert_eq!(company.first_name, result.data.first_name);
        assert_eq!(company.last_name, result.data.last_name);
    }
}

mod delete {
    use crate::common::{data, process, setup};
    use actix_web::{http::header::AUTHORIZATION, test};
    use framerate::{company::delete, utils::response_body::DeleteResponse};

    #[actix_web::test]
    async fn should_require_authentication() {
        let (app, pool) = setup::create_app(delete).await;

        let company = {
            let mut conn = pool.get().unwrap();
            let (_, user) = data::create_authed_user(&mut conn);
            data::create_company(&mut conn, &user)
        };

        let request = test::TestRequest::delete()
            .uri(&format!("/company/{}", company.company_id))
            .to_request();

        let response = test::call_service(&app, request).await;
        assert_eq!(401, response.status());
    }

    #[actix_web::test]
    async fn should_not_delete_other_users_company() {
        let (app, pool) = setup::create_app(delete).await;

        let (token, company) = {
            let mut conn = pool.get().unwrap();
            let (token, _) = data::create_authed_user(&mut conn);
            let other_user = data::create_user(&mut conn);
            let company = data::create_company(&mut conn, &other_user);

            (token, company)
        };

        let request = test::TestRequest::delete()
            .uri(&format!("/company/{}", company.company_id))
            .insert_header((AUTHORIZATION, format!("Bearer {token}")))
            .to_request();

        let response = test::call_service(&app, request).await;
        assert_eq!(404, response.status());
    }

    #[actix_web::test]
    async fn should_delete_company() {
        let (app, pool) = setup::create_app(delete).await;

        let (token, company) = {
            let mut conn = pool.get().unwrap();
            let (token, user) = data::create_authed_user(&mut conn);
            let company = data::create_company(&mut conn, &user);

            (token, company)
        };

        let request = test::TestRequest::delete()
            .uri(&format!("/company/{}", company.company_id))
            .insert_header((AUTHORIZATION, format!("Bearer {token}")))
            .to_request();

        let response = test::call_service(&app, request).await;
        assert!(response.status().is_success());

        let result = process::parse_body::<DeleteResponse>(response).await;
        assert_eq!(1, result.data.count);
    }
}
