mod common;

mod find {
    use crate::common::{data, process, setup};
    use actix_web::{http::header::AUTHORIZATION, test};
    use framerate::{
        movie_entry::{find, MovieEntry},
        watchlist::Watchlist,
    };

    #[actix_web::test]
    async fn should_require_authentication() {
        let (app, pool) = setup::create_app(find).await;
        let movie_entry = {
            let mut conn = pool.get().unwrap();
            let user = data::create_user(&mut conn);

            let watchlist =
                Watchlist::create(&mut conn, data::generate_movie_watchlist(user.user_id)).unwrap();
            MovieEntry::create(
                &mut conn,
                data::generate_movie_entry(user.user_id, watchlist.watchlist_id),
            )
            .unwrap()
        };

        let request = test::TestRequest::get()
            .uri(&format!(
                "/movies/entries/{}/{}",
                movie_entry.watchlist_id, movie_entry.movie_id
            ))
            .to_request();

        let response = test::call_service(&app, request).await;
        assert_eq!(401, response.status());
    }

    #[actix_web::test]
    async fn should_not_return_other_users_movie_entry() {
        let (app, pool) = setup::create_app(find).await;
        let (token, movie_entry) = {
            let mut conn = pool.get().unwrap();
            let user = data::create_user(&mut conn);

            let watchlist =
                Watchlist::create(&mut conn, data::generate_movie_watchlist(user.user_id)).unwrap();
            let movie_entry = MovieEntry::create(
                &mut conn,
                data::generate_movie_entry(user.user_id, watchlist.watchlist_id),
            )
            .unwrap();

            let (token, _) = data::create_authed_user(&mut conn);

            (token, movie_entry)
        };

        let request = test::TestRequest::get()
            .uri(&format!(
                "/movies/entries/{}/{}",
                movie_entry.watchlist_id, movie_entry.movie_id
            ))
            .insert_header((AUTHORIZATION, format!("Bearer {token}")))
            .to_request();

        let response = test::call_service(&app, request).await;
        assert_eq!(404, response.status());
    }

    #[actix_web::test]
    async fn should_return_movie_entry() {
        let (app, pool) = setup::create_app(find).await;
        let (token, movie_entry) = {
            let mut conn = pool.get().unwrap();
            let (token, user) = data::create_authed_user(&mut conn);

            let watchlist =
                Watchlist::create(&mut conn, data::generate_movie_watchlist(user.user_id)).unwrap();
            let movie_entry = MovieEntry::create(
                &mut conn,
                data::generate_movie_entry(user.user_id, watchlist.watchlist_id),
            )
            .unwrap();

            (token, movie_entry)
        };

        let request = test::TestRequest::get()
            .uri(&format!(
                "/movies/entries/{}/{}",
                movie_entry.watchlist_id, movie_entry.movie_id
            ))
            .insert_header((AUTHORIZATION, format!("Bearer {token}")))
            .to_request();

        let response = test::call_service(&app, request).await;
        assert!(response.status().is_success());

        let result = process::parse_body::<MovieEntry>(response).await;

        let returned_movie_entry = result.data;
        assert_eq!(movie_entry, returned_movie_entry);
    }
}

mod find_all {
    use crate::common::{data, process, setup};
    use actix_web::{http::header::AUTHORIZATION, test};
    use framerate::{
        movie_entry::{find_all, MovieEntry},
        watchlist::Watchlist,
    };

    #[actix_web::test]
    async fn should_require_authentication() {
        let (app, pool) = setup::create_app(find_all).await;
        let movie_entry = {
            let mut conn = pool.get().unwrap();
            let user = data::create_user(&mut conn);

            let watchlist =
                Watchlist::create(&mut conn, data::generate_movie_watchlist(user.user_id)).unwrap();
            MovieEntry::create(
                &mut conn,
                data::generate_movie_entry(user.user_id, watchlist.watchlist_id),
            )
            .unwrap()
        };

        let request = test::TestRequest::get()
            .uri(&format!("/movies/entries/{}", movie_entry.watchlist_id))
            .to_request();

        let response = test::call_service(&app, request).await;
        assert_eq!(401, response.status());
    }

    #[actix_web::test]
    async fn should_not_return_other_users_movie_entries() {
        let (app, pool) = setup::create_app(find_all).await;
        let (token, watchlist) = {
            let mut conn = pool.get().unwrap();
            let user = data::create_user(&mut conn);

            let watchlist =
                Watchlist::create(&mut conn, data::generate_movie_watchlist(user.user_id)).unwrap();
            MovieEntry::create(
                &mut conn,
                data::generate_movie_entry(user.user_id, watchlist.watchlist_id),
            )
            .unwrap();

            let (token, _) = data::create_authed_user(&mut conn);

            (token, watchlist)
        };

        let request = test::TestRequest::get()
            .uri(&format!("/movies/entries/{}", watchlist.watchlist_id))
            .insert_header((AUTHORIZATION, format!("Bearer {token}")))
            .to_request();

        let response = test::call_service(&app, request).await;
        assert!(response.status().is_success());

        let result = process::parse_body::<Vec<Watchlist>>(response).await;
        assert_eq!(0, result.data.len());
    }

    #[actix_web::test]
    async fn should_return_user_movie_entries() {
        let (app, pool) = setup::create_app(find_all).await;
        let (token, movie_entry) = {
            let mut conn = pool.get().unwrap();
            let (token, user) = data::create_authed_user(&mut conn);

            let watchlist =
                Watchlist::create(&mut conn, data::generate_movie_watchlist(user.user_id)).unwrap();
            let movie_entry = MovieEntry::create(
                &mut conn,
                data::generate_movie_entry(user.user_id, watchlist.watchlist_id),
            )
            .unwrap();

            (token, movie_entry)
        };

        let request = test::TestRequest::get()
            .uri(&format!("/movies/entries/{}", movie_entry.watchlist_id))
            .insert_header((AUTHORIZATION, format!("Bearer {token}")))
            .to_request();

        let response = test::call_service(&app, request).await;
        assert!(response.status().is_success());

        let result = process::parse_body::<Vec<MovieEntry>>(response).await;
        assert_eq!(1, result.data.len());

        let returned_movie_entry = result.data.first().unwrap();
        assert_eq!(&movie_entry, returned_movie_entry);
    }
}

mod create {
    use crate::common::{data, process, setup};
    use actix_web::{http::header::AUTHORIZATION, test};
    use framerate::{
        movie_entry::{create, MovieEntry},
        watchlist::Watchlist,
    };

    #[actix_web::test]
    async fn should_require_authentication() {
        let (app, pool) = setup::create_app(create).await;
        let watchlist = {
            let mut conn = pool.get().unwrap();
            let user = data::create_user(&mut conn);

            let watchlist =
                Watchlist::create(&mut conn, data::generate_movie_watchlist(user.user_id)).unwrap();

            watchlist
        };

        let movie_entry = data::generate_save_movie_entry();

        let request = test::TestRequest::post()
            .uri(&format!("/movies/entries/{}", watchlist.watchlist_id))
            .set_json(&movie_entry)
            .to_request();

        let response = test::call_service(&app, request).await;
        assert_eq!(401, response.status());
    }

    #[actix_web::test]
    async fn should_not_create_movie_entry_on_other_users_watchlist() {
        let (app, pool) = setup::create_app(create).await;
        let (token, watchlist) = {
            let mut conn = pool.get().unwrap();
            let user = data::create_user(&mut conn);

            let watchlist =
                Watchlist::create(&mut conn, data::generate_movie_watchlist(user.user_id)).unwrap();

            let (token, _) = data::create_authed_user(&mut conn);

            (token, watchlist)
        };

        let movie_entry = data::generate_save_movie_entry();

        let request = test::TestRequest::post()
            .uri(&format!("/movies/entries/{}", watchlist.watchlist_id))
            .set_json(&movie_entry)
            .insert_header((AUTHORIZATION, format!("Bearer {token}")))
            .to_request();

        let response = test::call_service(&app, request).await;
        assert!(response.status().is_success());

        let result = process::parse_body::<MovieEntry>(response).await;

        // This will generate a new watchlist for the user, so check that the other watchlist id is not used
        assert_ne!(watchlist.watchlist_id, result.data.watchlist_id);
    }

    #[actix_web::test]
    async fn should_create_movie_entry() {
        let (app, pool) = setup::create_app(create).await;
        let (token, user, watchlist) = {
            let mut conn = pool.get().unwrap();
            let (token, user) = data::create_authed_user(&mut conn);

            let watchlist =
                Watchlist::create(&mut conn, data::generate_movie_watchlist(user.user_id)).unwrap();

            (token, user, watchlist)
        };

        let movie_entry = data::generate_save_movie_entry();

        let request = test::TestRequest::post()
            .uri(&format!("/movies/entries/{}", watchlist.watchlist_id))
            .set_json(&movie_entry)
            .insert_header((AUTHORIZATION, format!("Bearer {token}")))
            .to_request();

        let response = test::call_service(&app, request).await;
        assert!(response.status().is_success());

        let result = process::parse_body::<MovieEntry>(response).await;
        let returned_movie_entry = result.data;

        assert!(!returned_movie_entry.watchlist_id.is_nil());
        assert_eq!(movie_entry.movie_id, returned_movie_entry.movie_id);
        assert_eq!(user.user_id, returned_movie_entry.user_id);
    }
}

mod delete {
    use crate::common::{data, process, setup};
    use actix_web::{http::header::AUTHORIZATION, test};
    use framerate::{
        movie_entry::{delete, DeleteResponse, MovieEntry},
        watchlist::Watchlist,
    };

    #[actix_web::test]
    async fn should_require_authentication() {
        let (app, pool) = setup::create_app(delete).await;
        let movie_entry = {
            let mut conn = pool.get().unwrap();
            let user = data::create_user(&mut conn);

            let watchlist =
                Watchlist::create(&mut conn, data::generate_movie_watchlist(user.user_id)).unwrap();

            MovieEntry::create(
                &mut conn,
                data::generate_movie_entry(user.user_id, watchlist.watchlist_id),
            )
            .unwrap()
        };

        let request = test::TestRequest::delete()
            .uri(&format!(
                "/movies/entries/{}/{}",
                movie_entry.watchlist_id, movie_entry.movie_id
            ))
            .to_request();

        let response = test::call_service(&app, request).await;
        assert_eq!(401, response.status());
    }

    #[actix_web::test]
    async fn should_not_delete_other_users_movie_entry() {
        let (app, pool) = setup::create_app(delete).await;
        let (token, movie_entry) = {
            let mut conn = pool.get().unwrap();
            let user = data::create_user(&mut conn);

            let watchlist =
                Watchlist::create(&mut conn, data::generate_movie_watchlist(user.user_id)).unwrap();

            let movie_entry = MovieEntry::create(
                &mut conn,
                data::generate_movie_entry(user.user_id, watchlist.watchlist_id),
            )
            .unwrap();

            let (token, _) = data::create_authed_user(&mut conn);

            (token, movie_entry)
        };

        let request = test::TestRequest::delete()
            .uri(&format!(
                "/movies/entries/{}/{}",
                movie_entry.watchlist_id, movie_entry.movie_id
            ))
            .insert_header((AUTHORIZATION, format!("Bearer {token}")))
            .to_request();

        let response = test::call_service(&app, request).await;
        assert_eq!(404, response.status());
    }

    #[actix_web::test]
    async fn should_delete_movie_entry() {
        let (app, pool) = setup::create_app(delete).await;
        let (token, movie_entry) = {
            let mut conn = pool.get().unwrap();
            let (token, user) = data::create_authed_user(&mut conn);

            let watchlist =
                Watchlist::create(&mut conn, data::generate_movie_watchlist(user.user_id)).unwrap();

            let movie_entry = MovieEntry::create(
                &mut conn,
                data::generate_movie_entry(user.user_id, watchlist.watchlist_id),
            )
            .unwrap();

            (token, movie_entry)
        };

        let request = test::TestRequest::delete()
            .uri(&format!(
                "/movies/entries/{}/{}",
                movie_entry.watchlist_id, movie_entry.movie_id
            ))
            .insert_header((AUTHORIZATION, format!("Bearer {token}")))
            .to_request();

        let response = test::call_service(&app, request).await;
        assert!(response.status().is_success());

        let result = process::parse_body::<DeleteResponse>(response).await;

        assert_eq!(1, result.data.count);
    }
}
