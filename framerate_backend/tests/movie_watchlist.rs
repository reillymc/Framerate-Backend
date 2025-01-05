pub mod common;

mod find {
    use crate::common::{data, process, setup};
    use actix_web::{http::header::AUTHORIZATION, test};
    use framerate::movie_watchlist::{find, MovieWatchlist};

    #[actix_web::test]
    async fn should_require_authentication() {
        let (app, pool) = setup::create_app(find).await;

        {
            let mut conn = pool.get().unwrap();
            let user = data::create_user(&mut conn);
            data::create_default_movie_watchlist(&mut conn, &user)
        };

        let request = test::TestRequest::get()
            .uri("/movies/watchlist")
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
            let watchlist = data::create_default_movie_watchlist(&mut conn, &user);
            let (token, _) = data::create_authed_user(&mut conn);
            (token, watchlist)
        };

        let request: actix_http::Request = test::TestRequest::get()
            .uri("/movies/watchlist")
            .insert_header((AUTHORIZATION, format!("Bearer {token}")))
            .to_request();

        let response = test::call_service(&app, request).await;
        assert!(response.status().is_success());

        let result = process::parse_body::<MovieWatchlist>(response).await;

        // This will generate a new watchlist for the user, so check that the existing watchlist is not returned
        assert_ne!(watchlist.name, result.data.name);
    }

    #[actix_web::test]
    async fn should_return_watchlist() {
        let (app, pool) = setup::create_app(find).await;

        let (token, watchlist) = {
            let mut conn = pool.get().unwrap();
            let (token, user) = data::create_authed_user(&mut conn);
            let watchlist = data::create_default_movie_watchlist(&mut conn, &user);
            (token, watchlist)
        };

        let request = test::TestRequest::get()
            .uri("/movies/watchlist")
            .insert_header((AUTHORIZATION, format!("Bearer {token}")))
            .to_request();

        let response = test::call_service(&app, request).await;
        assert!(response.status().is_success());

        let result = process::parse_body::<MovieWatchlist>(response).await;

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
            .uri("/movies/watchlist")
            .insert_header((AUTHORIZATION, format!("Bearer {token}")))
            .to_request();

        let response = test::call_service(&app, request).await;
        assert!(response.status().is_success());

        let result = process::parse_body::<MovieWatchlist>(response).await;

        let returned_watchlist = result.data;
        assert_eq!("Movie Watchlist", returned_watchlist.name);
    }
}

mod find_entry {
    use crate::common::{data, process, setup};
    use actix_web::{http::header::AUTHORIZATION, test};
    use framerate::movie_watchlist::{find_entry, MovieWatchlistEntry};

    #[actix_web::test]
    async fn should_require_authentication() {
        let (app, pool) = setup::create_app(find_entry).await;

        let movie_entry = {
            let mut conn = pool.get().unwrap();
            let user = data::create_user(&mut conn);
            let watchlist = data::create_default_movie_watchlist(&mut conn, &user);
            data::create_movie_entry(&mut conn, &user, &watchlist)
        };

        let request = test::TestRequest::get()
            .uri(&format!("/movies/watchlist/{}", movie_entry.movie_id))
            .to_request();

        let response = test::call_service(&app, request).await;
        assert_eq!(401, response.status());
    }

    #[actix_web::test]
    async fn should_not_return_other_users_movie_entry() {
        let (app, pool) = setup::create_app(find_entry).await;

        let (token, movie_entry) = {
            let mut conn = pool.get().unwrap();
            let user = data::create_user(&mut conn);
            let watchlist = data::create_default_movie_watchlist(&mut conn, &user);
            let movie_entry = data::create_movie_entry(&mut conn, &user, &watchlist);
            let (token, _) = data::create_authed_user(&mut conn);
            (token, movie_entry)
        };

        let request = test::TestRequest::get()
            .uri(&format!("/movies/watchlist/{}", movie_entry.movie_id))
            .insert_header((AUTHORIZATION, format!("Bearer {token}")))
            .to_request();

        let response = test::call_service(&app, request).await;
        assert_eq!(404, response.status());
    }

    #[actix_web::test]
    async fn should_return_movie_entry() {
        let (app, pool) = setup::create_app(find_entry).await;

        let (token, movie_entry) = {
            let mut conn = pool.get().unwrap();
            let (token, user) = data::create_authed_user(&mut conn);
            let watchlist = data::create_default_movie_watchlist(&mut conn, &user);
            let movie_entry = data::create_movie_entry(&mut conn, &user, &watchlist);
            (token, movie_entry)
        };

        let request = test::TestRequest::get()
            .uri(&format!("/movies/watchlist/{}", movie_entry.movie_id))
            .insert_header((AUTHORIZATION, format!("Bearer {token}")))
            .to_request();

        let response = test::call_service(&app, request).await;
        assert!(response.status().is_success());

        let result = process::parse_body::<MovieWatchlistEntry>(response).await;

        let returned_movie_entry = result.data;
        assert_eq!(movie_entry.title, returned_movie_entry.title);
        assert_eq!(movie_entry.imdb_id, returned_movie_entry.imdb_id);
        assert_eq!(movie_entry.release_date, returned_movie_entry.release_date);
        assert_eq!(movie_entry.poster_path, returned_movie_entry.poster_path);
        assert_eq!(movie_entry.movie_id, returned_movie_entry.movie_id);
        assert_eq!(movie_entry.status, returned_movie_entry.status);
        assert_eq!(movie_entry.updated_at, returned_movie_entry.updated_at);
    }
}

mod create_entry {
    use crate::common::{data, process, setup};
    use actix_web::{http::header::AUTHORIZATION, test};
    use framerate::movie_watchlist::{create_entry, MovieWatchlistEntry};

    #[actix_web::test]
    async fn should_require_authentication() {
        let (app, pool) = setup::create_app(create_entry).await;

        {
            let mut conn = pool.get().unwrap();
            let user = data::create_user(&mut conn);
            data::create_default_movie_watchlist(&mut conn, &user)
        };

        let movie_entry = data::generate_save_movie_watchlist_entry();

        let request = test::TestRequest::post()
            .uri("/movies/watchlist")
            .set_json(&movie_entry)
            .to_request();

        let response = test::call_service(&app, request).await;
        assert_eq!(401, response.status());
    }

    #[actix_web::test]
    async fn should_create_movie_entry() {
        let (app, pool) = setup::create_app(create_entry).await;

        let token = {
            let mut conn = pool.get().unwrap();
            let (token, user) = data::create_authed_user(&mut conn);
            data::create_default_movie_watchlist(&mut conn, &user);
            token
        };

        let movie_entry = data::generate_save_movie_watchlist_entry();

        let request = test::TestRequest::post()
            .uri("/movies/watchlist")
            .set_json(&movie_entry)
            .insert_header((AUTHORIZATION, format!("Bearer {token}")))
            .to_request();

        let response = test::call_service(&app, request).await;
        assert!(response.status().is_success());

        let result = process::parse_body::<MovieWatchlistEntry>(response).await;
        let returned_movie_entry = result.data;

        assert_eq!(movie_entry.movie_id, returned_movie_entry.movie_id);
    }
}

mod delete_entry {
    use crate::common::{data, process, setup};
    use actix_web::{http::header::AUTHORIZATION, test};
    use framerate::{movie_watchlist::delete_entry, utils::response_body::DeleteResponse};

    #[actix_web::test]
    async fn should_require_authentication() {
        let (app, pool) = setup::create_app(delete_entry).await;

        let movie_entry = {
            let mut conn = pool.get().unwrap();
            let user = data::create_user(&mut conn);
            let watchlist = data::create_default_movie_watchlist(&mut conn, &user);
            data::create_movie_entry(&mut conn, &user, &watchlist)
        };

        let request = test::TestRequest::delete()
            .uri(&format!("/movies/watchlist/{}", movie_entry.movie_id))
            .to_request();

        let response = test::call_service(&app, request).await;
        assert_eq!(401, response.status());
    }

    #[actix_web::test]
    async fn should_not_delete_other_users_movie_entry() {
        let (app, pool) = setup::create_app(delete_entry).await;

        let (token, movie_entry) = {
            let mut conn = pool.get().unwrap();
            let user = data::create_user(&mut conn);
            let watchlist = data::create_default_movie_watchlist(&mut conn, &user);
            let movie_entry = data::create_movie_entry(&mut conn, &user, &watchlist);
            let (token, _) = data::create_authed_user(&mut conn);
            (token, movie_entry)
        };

        let request = test::TestRequest::delete()
            .uri(&format!("/movies/watchlist/{}", movie_entry.movie_id))
            .insert_header((AUTHORIZATION, format!("Bearer {token}")))
            .to_request();

        let response = test::call_service(&app, request).await;
        assert_eq!(404, response.status());
    }

    #[actix_web::test]
    async fn should_delete_movie_entry() {
        let (app, pool) = setup::create_app(delete_entry).await;

        let (token, movie_entry) = {
            let mut conn = pool.get().unwrap();
            let (token, user) = data::create_authed_user(&mut conn);
            let watchlist = data::create_default_movie_watchlist(&mut conn, &user);
            let movie_entry = data::create_movie_entry(&mut conn, &user, &watchlist);
            (token, movie_entry)
        };

        let request = test::TestRequest::delete()
            .uri(&format!("/movies/watchlist/{}", movie_entry.movie_id))
            .insert_header((AUTHORIZATION, format!("Bearer {token}")))
            .to_request();

        let response = test::call_service(&app, request).await;
        assert!(response.status().is_success());

        let result = process::parse_body::<DeleteResponse>(response).await;

        assert_eq!(1, result.data.count);
    }
}
