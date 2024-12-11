mod common;

mod search {
    use crate::common::{data, process, setup};
    use actix_web::{http::header::AUTHORIZATION, test};
    use framerate::show::{search, Show};

    #[actix_web::test]
    async fn should_require_authentication() {
        let (app, _) = setup::create_app(search).await;

        let request = test::TestRequest::get()
            .uri("/shows/search?query=Doctor%20Who")
            .to_request();

        let response = test::call_service(&app, request).await;
        assert!(response.status().is_client_error());
    }

    #[actix_web::test]
    async fn should_return_results() {
        let (app, pool) = setup::create_app(search).await;
        let (token, _) = {
            let mut conn = pool.get().unwrap();
            data::create_authed_user(&mut conn)
        };

        let request = test::TestRequest::get()
            .uri("/shows/search?query=Doctor%20Who")
            .insert_header((AUTHORIZATION, format!("Bearer {token}")))
            .to_request();

        let response = test::call_service(&app, request).await;
        assert!(response.status().is_success());

        let result = process::parse_body::<Vec<Show>>(response).await;
        assert!(result.data.len() > 0);
    }
}

mod popular {
    use crate::common::{data, process, setup};
    use actix_web::{http::header::AUTHORIZATION, test};
    use framerate::show::{popular, Show};

    #[actix_web::test]
    async fn should_require_authentication() {
        let (app, _) = setup::create_app(popular).await;

        let request = test::TestRequest::get().uri("/shows/popular").to_request();

        let response = test::call_service(&app, request).await;
        assert_eq!(401, response.status());
    }

    #[actix_web::test]
    async fn should_return_results() {
        let (app, pool) = setup::create_app(popular).await;
        let (token, _) = {
            let mut conn = pool.get().unwrap();
            data::create_authed_user(&mut conn)
        };

        let request = test::TestRequest::get()
            .uri("/shows/popular")
            .insert_header((AUTHORIZATION, format!("Bearer {token}")))
            .to_request();

        let response = test::call_service(&app, request).await;
        assert!(response.status().is_success());

        let result = process::parse_body::<Vec<Show>>(response).await;
        assert!(result.data.len() > 0);
    }
}

mod details {
    use crate::common::{data, process, setup};
    use actix_web::{http::header::AUTHORIZATION, test};
    use framerate::{show::details, show::Show};

    #[actix_web::test]
    async fn should_require_authentication() {
        let (app, _) = setup::create_app(details).await;

        let request = test::TestRequest::get()
            .uri("/shows/57243/details")
            .to_request();

        let response = test::call_service(&app, request).await;
        assert_eq!(401, response.status());
    }

    #[actix_web::test]
    async fn should_return_show() {
        let (app, pool) = setup::create_app(details).await;
        let (token, _) = {
            let mut conn = pool.get().unwrap();
            data::create_authed_user(&mut conn)
        };

        let request = test::TestRequest::get()
            .uri("/shows/57243/details")
            .insert_header((AUTHORIZATION, format!("Bearer {token}")))
            .to_request();

        let response = test::call_service(&app, request).await;
        assert!(response.status().is_success());

        let result = process::parse_body::<Show>(response).await;
        assert_eq!(result.data.id, 57243);

        let seasons = &result.data.seasons.unwrap();
        assert!(seasons.len() > 0);
    }
}
