pub mod common;

mod find_all {
    use crate::common::{data, process, setup};
    use actix_web::{http::header::AUTHORIZATION, test};
    use framerate::watchlist::{find_all, Watchlist};

    #[actix_web::test]
    async fn should_require_authentication() {
        let (app, _) = setup::create_app(find_all).await;

        let request = test::TestRequest::get().uri("/watchlists").to_request();

        let response = test::call_service(&app, request).await;
        assert_eq!(401, response.status());
    }

    #[actix_web::test]
    async fn should_not_return_other_users_watchlists() {
        let (app, pool) = setup::create_app(find_all).await;
        let token = {
            let mut conn = pool.get().unwrap();
            let user = data::create_user(&mut conn);

            Watchlist::create(&mut conn, data::generate_watchlist(user.user_id)).unwrap();

            let (token, _) = data::create_authed_user(&mut conn);

            token
        };

        let request = test::TestRequest::get()
            .uri("/watchlists")
            .insert_header((AUTHORIZATION, format!("Bearer {token}")))
            .to_request();

        let response = test::call_service(&app, request).await;
        assert!(response.status().is_success());

        let result = process::parse_body::<Vec<Watchlist>>(response).await;
        assert_eq!(0, result.data.len());
    }

    #[actix_web::test]
    async fn should_return_user_watchlists() {
        let (app, pool) = setup::create_app(find_all).await;
        let (user, token, watchlist) = {
            let mut conn = pool.get().unwrap();
            let (token, user) = data::create_authed_user(&mut conn);

            let watchlist =
                Watchlist::create(&mut conn, data::generate_watchlist(user.user_id)).unwrap();

            (user, token, watchlist)
        };

        let request = test::TestRequest::get()
            .uri("/watchlists")
            .insert_header((AUTHORIZATION, format!("Bearer {token}")))
            .to_request();

        let response = test::call_service(&app, request).await;
        assert!(response.status().is_success());

        let result = process::parse_body::<Vec<Watchlist>>(response).await;
        assert_eq!(1, result.data.len());

        let returned_watchlist = result.data.first().unwrap();
        assert_eq!(watchlist.watchlist_id, returned_watchlist.watchlist_id);
        assert_eq!(user.user_id, returned_watchlist.user_id);
        assert_eq!(watchlist.media_type, returned_watchlist.media_type);
        assert_eq!(watchlist.name, returned_watchlist.name);
    }
}

mod find_by_media_type {
    use crate::common::{data, process, setup};
    use actix_web::{http::header::AUTHORIZATION, test};
    use framerate::watchlist::{find_by_media_type, Watchlist};

    #[actix_web::test]
    async fn should_require_authentication() {
        let (app, pool) = setup::create_app(find_by_media_type).await;
        let watchlist = {
            let mut conn = pool.get().unwrap();
            let user = data::create_user(&mut conn);

            Watchlist::create(&mut conn, data::generate_watchlist(user.user_id)).unwrap()
        };

        let request = test::TestRequest::get()
            .uri(&format!("/watchlists/{}", watchlist.media_type))
            .to_request();

        let response = test::call_service(&app, request).await;
        assert_eq!(401, response.status());
    }

    #[actix_web::test]
    async fn should_not_return_other_users_watchlists() {
        let (app, pool) = setup::create_app(find_by_media_type).await;
        let (token, watchlist) = {
            let mut conn = pool.get().unwrap();
            let user = data::create_user(&mut conn);

            let watchlist =
                Watchlist::create(&mut conn, data::generate_watchlist(user.user_id)).unwrap();

            let (token, _) = data::create_authed_user(&mut conn);

            (token, watchlist)
        };

        let request = test::TestRequest::get()
            .uri(&format!("/watchlists/{}", watchlist.media_type))
            .insert_header((AUTHORIZATION, format!("Bearer {token}")))
            .to_request();

        let response = test::call_service(&app, request).await;
        assert!(response.status().is_success());

        let result = process::parse_body::<Watchlist>(response).await;

        // This will generate a new watchlist for the user, so check that the existing watchlist is not returned
        assert_ne!(watchlist.watchlist_id, result.data.watchlist_id);
    }

    #[actix_web::test]
    async fn should_return_watchlists() {
        let (app, pool) = setup::create_app(find_by_media_type).await;
        let (user, token, watchlist) = {
            let mut conn = pool.get().unwrap();
            let (token, user) = data::create_authed_user(&mut conn);

            let watchlist =
                Watchlist::create(&mut conn, data::generate_watchlist(user.user_id)).unwrap();

            (user, token, watchlist)
        };

        let request = test::TestRequest::get()
            .uri(&format!("/watchlists/{}", watchlist.media_type))
            .insert_header((AUTHORIZATION, format!("Bearer {token}")))
            .to_request();

        let response = test::call_service(&app, request).await;
        assert!(response.status().is_success());

        let result = process::parse_body::<Watchlist>(response).await;

        let returned_watchlist = result.data;
        assert_eq!(watchlist.watchlist_id, returned_watchlist.watchlist_id);
        assert_eq!(user.user_id, returned_watchlist.user_id);
        assert_eq!(watchlist.media_type, returned_watchlist.media_type);
        assert_eq!(watchlist.name, returned_watchlist.name);
    }

    #[actix_web::test]
    async fn should_return_new_watchlist_when_none_exist() {
        let (app, pool) = setup::create_app(find_by_media_type).await;
        let (user, token) = {
            let mut conn = pool.get().unwrap();
            let (token, user) = data::create_authed_user(&mut conn);

            (user, token)
        };

        let media_type = data::generate_watchlist(user.user_id).media_type;

        let request = test::TestRequest::get()
            .uri(&format!("/watchlists/{}", media_type))
            .insert_header((AUTHORIZATION, format!("Bearer {token}")))
            .to_request();

        let response = test::call_service(&app, request).await;
        assert!(response.status().is_success());

        let result = process::parse_body::<Watchlist>(response).await;

        let expected_name = match media_type.as_str() {
            "movie" => "Movie Watchlist",
            "show" => "Show Watchlist",
            _ => "Watchlist",
        }
        .to_string();

        let returned_watchlist = result.data;
        assert!(!returned_watchlist.watchlist_id.is_nil());
        assert_eq!(user.user_id, returned_watchlist.user_id);
        assert_eq!(media_type, returned_watchlist.media_type);
        assert_eq!(expected_name, returned_watchlist.name);
    }
}

mod create {
    use crate::common::{data, process, setup};
    use actix_web::{http::header::AUTHORIZATION, test};
    use framerate::watchlist::{create, Watchlist};

    #[actix_web::test]
    async fn should_require_authentication() {
        let (app, _) = setup::create_app(create).await;

        let request = test::TestRequest::post().uri("/watchlists").to_request();

        let response = test::call_service(&app, request).await;
        assert_eq!(401, response.status());
    }

    #[actix_web::test]
    async fn should_create_watchlist() {
        let (app, pool) = setup::create_app(create).await;
        let (token, user) = {
            let mut conn = pool.get().unwrap();
            let (token, user) = data::create_authed_user(&mut conn);
            (token, user)
        };

        let watchlist = data::generate_new_watchlist();

        let request = test::TestRequest::post()
            .uri("/watchlists")
            .set_json(&watchlist)
            .insert_header((AUTHORIZATION, format!("Bearer {token}")))
            .to_request();

        let response = test::call_service(&app, request).await;
        assert!(response.status().is_success());

        let result = process::parse_body::<Watchlist>(response).await;
        let returned_watchlist = result.data;

        assert!(!returned_watchlist.watchlist_id.is_nil());
        assert_eq!(user.user_id, returned_watchlist.user_id);
        assert_eq!(watchlist.media_type, returned_watchlist.media_type);
        assert_eq!(watchlist.name, returned_watchlist.name);
    }
}
