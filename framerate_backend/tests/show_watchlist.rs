pub mod common;

mod find {
    use crate::common::{data, process, setup};
    use actix_web::{http::header::AUTHORIZATION, test};
    use framerate::show_watchlist::{find, ShowWatchlist};

    #[actix_web::test]
    async fn should_require_authentication() {
        let (app, pool) = setup::create_app(find).await;

        {
            let mut conn = pool.get().unwrap();
            let user = data::create_user(&mut conn);
            data::create_default_show_watchlist(&mut conn, &user)
        };

        let request = test::TestRequest::get()
            .uri("/shows/watchlist")
            .to_request();

        let response = test::call_service(&app, request).await;
        assert_eq!(401, response.status());
    }

    #[actix_web::test]
    async fn should_not_return_other_users_watchlist() {
        let (app, pool) = setup::create_app(find).await;

        let (token, watchlist) = {
            let mut conn = pool.get().unwrap();
            let user = data::create_user(&mut conn);
            let watchlist = data::create_default_show_watchlist(&mut conn, &user);
            let (token, _) = data::create_authed_user(&mut conn);
            (token, watchlist)
        };

        let request: actix_http::Request = test::TestRequest::get()
            .uri("/shows/watchlist")
            .insert_header((AUTHORIZATION, format!("Bearer {token}")))
            .to_request();

        let response = test::call_service(&app, request).await;
        assert!(response.status().is_success());

        let result = process::parse_body::<ShowWatchlist>(response).await;

        // This will generate a new watchlist for the user, so check that the existing watchlist is not returned
        assert_ne!(watchlist.name, result.data.name);
    }

    #[actix_web::test]
    async fn should_return_watchlist() {
        let (app, pool) = setup::create_app(find).await;

        let (token, watchlist) = {
            let mut conn = pool.get().unwrap();
            let (token, user) = data::create_authed_user(&mut conn);
            let watchlist = data::create_default_show_watchlist(&mut conn, &user);
            (token, watchlist)
        };

        let request = test::TestRequest::get()
            .uri("/shows/watchlist")
            .insert_header((AUTHORIZATION, format!("Bearer {token}")))
            .to_request();

        let response = test::call_service(&app, request).await;
        assert!(response.status().is_success());

        let result = process::parse_body::<ShowWatchlist>(response).await;

        let returned_watchlist = result.data;
        assert_eq!(watchlist.name, returned_watchlist.name);
    }

    #[actix_web::test]
    async fn should_return_new_default_watchlist_when_none_exist() {
        let (app, pool) = setup::create_app(find).await;

        let token = {
            let mut conn = pool.get().unwrap();
            let (token, _) = data::create_authed_user(&mut conn);
            token
        };

        let request = test::TestRequest::get()
            .uri("/shows/watchlist")
            .insert_header((AUTHORIZATION, format!("Bearer {token}")))
            .to_request();

        let response = test::call_service(&app, request).await;
        assert!(response.status().is_success());

        let result = process::parse_body::<ShowWatchlist>(response).await;

        let returned_watchlist = result.data;
        assert_eq!("Show Watchlist", returned_watchlist.name);
    }
}

mod find_entry {
    use crate::common::{data, process, setup};
    use actix_web::{http::header::AUTHORIZATION, test};
    use framerate::show_watchlist::{find_entry, ShowWatchlistEntry};

    #[actix_web::test]
    async fn should_require_authentication() {
        let (app, pool) = setup::create_app(find_entry).await;

        let show_entry = {
            let mut conn = pool.get().unwrap();
            let user = data::create_user(&mut conn);
            let watchlist = data::create_default_show_watchlist(&mut conn, &user);
            data::create_show_entry(&mut conn, &user, &watchlist)
        };

        let request = test::TestRequest::get()
            .uri(&format!("/shows/watchlist/{}", show_entry.show_id))
            .to_request();

        let response = test::call_service(&app, request).await;
        assert_eq!(401, response.status());
    }

    #[actix_web::test]
    async fn should_not_return_other_users_show_entry() {
        let (app, pool) = setup::create_app(find_entry).await;

        let (token, show_entry) = {
            let mut conn = pool.get().unwrap();
            let user = data::create_user(&mut conn);
            let watchlist = data::create_default_show_watchlist(&mut conn, &user);
            let show_entry = data::create_show_entry(&mut conn, &user, &watchlist);
            let (token, _) = data::create_authed_user(&mut conn);
            (token, show_entry)
        };

        let request = test::TestRequest::get()
            .uri(&format!("/shows/watchlist/{}", show_entry.show_id))
            .insert_header((AUTHORIZATION, format!("Bearer {token}")))
            .to_request();

        let response = test::call_service(&app, request).await;
        assert_eq!(404, response.status());
    }

    #[actix_web::test]
    async fn should_return_show_entry() {
        let (app, pool) = setup::create_app(find_entry).await;

        let (token, show_entry) = {
            let mut conn = pool.get().unwrap();
            let (token, user) = data::create_authed_user(&mut conn);
            let watchlist = data::create_default_show_watchlist(&mut conn, &user);
            let show_entry = data::create_show_entry(&mut conn, &user, &watchlist);
            (token, show_entry)
        };

        let request = test::TestRequest::get()
            .uri(&format!("/shows/watchlist/{}", show_entry.show_id))
            .insert_header((AUTHORIZATION, format!("Bearer {token}")))
            .to_request();

        let response = test::call_service(&app, request).await;
        assert!(response.status().is_success());

        let result = process::parse_body::<ShowWatchlistEntry>(response).await;

        let returned_show_entry = result.data;
        assert_eq!(
            show_entry.first_air_date,
            returned_show_entry.first_air_date
        );
        assert_eq!(show_entry.imdb_id, returned_show_entry.imdb_id);
        assert_eq!(show_entry.last_air_date, returned_show_entry.last_air_date);
        assert_eq!(show_entry.name, returned_show_entry.name);
        assert_eq!(show_entry.next_air_date, returned_show_entry.next_air_date);
        assert_eq!(show_entry.poster_path, returned_show_entry.poster_path);
        assert_eq!(show_entry.show_id, returned_show_entry.show_id);
        assert_eq!(show_entry.status, returned_show_entry.status);
        assert_eq!(show_entry.updated_at, returned_show_entry.updated_at);
    }
}

mod create_entry {
    use crate::common::{data, process, setup};
    use actix_web::{http::header::AUTHORIZATION, test};
    use framerate::show_watchlist::{create_entry, ShowWatchlistEntry};

    #[actix_web::test]
    async fn should_require_authentication() {
        let (app, pool) = setup::create_app(create_entry).await;

        {
            let mut conn = pool.get().unwrap();
            let user = data::create_user(&mut conn);
            data::create_default_show_watchlist(&mut conn, &user)
        };

        let show_entry = data::generate_save_show_watchlist_entry();

        let request = test::TestRequest::post()
            .uri("/shows/watchlist")
            .set_json(&show_entry)
            .to_request();

        let response = test::call_service(&app, request).await;
        assert_eq!(401, response.status());
    }

    #[actix_web::test]
    async fn should_create_show_entry() {
        let (app, pool) = setup::create_app(create_entry).await;

        let token = {
            let mut conn = pool.get().unwrap();
            let (token, user) = data::create_authed_user(&mut conn);
            data::create_default_show_watchlist(&mut conn, &user);
            token
        };

        let show_entry = data::generate_save_show_watchlist_entry();

        let request = test::TestRequest::post()
            .uri("/shows/watchlist")
            .set_json(&show_entry)
            .insert_header((AUTHORIZATION, format!("Bearer {token}")))
            .to_request();

        let response = test::call_service(&app, request).await;
        assert!(response.status().is_success());

        let result = process::parse_body::<ShowWatchlistEntry>(response).await;
        let returned_show_entry = result.data;

        assert_eq!(show_entry.show_id, returned_show_entry.show_id);
    }
}

mod delete_entry {
    use crate::common::{data, process, setup};
    use actix_web::{http::header::AUTHORIZATION, test};
    use framerate::{show_watchlist::delete_entry, utils::response_body::DeleteResponse};

    #[actix_web::test]
    async fn should_require_authentication() {
        let (app, pool) = setup::create_app(delete_entry).await;

        let show_entry = {
            let mut conn = pool.get().unwrap();
            let user = data::create_user(&mut conn);
            let watchlist = data::create_default_show_watchlist(&mut conn, &user);
            data::create_show_entry(&mut conn, &user, &watchlist)
        };

        let request = test::TestRequest::delete()
            .uri(&format!("/shows/watchlist/{}", show_entry.show_id))
            .to_request();

        let response = test::call_service(&app, request).await;
        assert_eq!(401, response.status());
    }

    #[actix_web::test]
    async fn should_not_delete_other_users_show_entry() {
        let (app, pool) = setup::create_app(delete_entry).await;

        let (token, show_entry) = {
            let mut conn = pool.get().unwrap();
            let user = data::create_user(&mut conn);
            let watchlist = data::create_default_show_watchlist(&mut conn, &user);
            let show_entry = data::create_show_entry(&mut conn, &user, &watchlist);
            let (token, _) = data::create_authed_user(&mut conn);
            (token, show_entry)
        };

        let request = test::TestRequest::delete()
            .uri(&format!("/shows/watchlist/{}", show_entry.show_id))
            .insert_header((AUTHORIZATION, format!("Bearer {token}")))
            .to_request();

        let response = test::call_service(&app, request).await;
        assert_eq!(404, response.status());
    }

    #[actix_web::test]
    async fn should_delete_show_entry() {
        let (app, pool) = setup::create_app(delete_entry).await;

        let (token, show_entry) = {
            let mut conn = pool.get().unwrap();
            let (token, user) = data::create_authed_user(&mut conn);
            let watchlist = data::create_default_show_watchlist(&mut conn, &user);
            let show_entry = data::create_show_entry(&mut conn, &user, &watchlist);
            (token, show_entry)
        };

        let request = test::TestRequest::delete()
            .uri(&format!("/shows/watchlist/{}", show_entry.show_id))
            .insert_header((AUTHORIZATION, format!("Bearer {token}")))
            .to_request();

        let response = test::call_service(&app, request).await;
        assert!(response.status().is_success());

        let result = process::parse_body::<DeleteResponse>(response).await;

        assert_eq!(1, result.data.count);
    }
}
