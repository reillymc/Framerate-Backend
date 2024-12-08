mod common;

mod find_all {
    use crate::common::{data, process, setup};
    use actix_web::{http::header::AUTHORIZATION, test};
    use framerate::{
        movie_review::find_all,
        movie_review::{MovieReview, MovieReviewResponse},
        review::Review,
    };

    #[actix_web::test]
    async fn should_require_authentication() {
        let (app, _) = setup::create_app(find_all).await;

        let request = test::TestRequest::get().uri("/movies/reviews").to_request();

        let response = test::call_service(&app, request).await;
        assert!(response.status().is_client_error());
    }

    #[actix_web::test]
    async fn should_not_return_other_users_reviews() {
        let (app, pool) = setup::create_app(find_all).await;
        let token = {
            let mut conn = pool.get().unwrap();
            let user = data::create_user(&mut conn);

            let review = Review::create(&mut conn, data::generate_review(user.user_id)).unwrap();
            MovieReview::create(
                &mut conn,
                data::generate_movie_review(user.user_id, review.review_id),
            )
            .unwrap();

            let (token, _) = data::create_authed_user(&mut conn);

            token
        };

        let request = test::TestRequest::get()
            .uri("/movies/reviews")
            .insert_header((AUTHORIZATION, format!("Bearer {token}")))
            .to_request();

        let response = test::call_service(&app, request).await;
        assert!(response.status().is_success());

        let result = process::parse_body::<Vec<MovieReviewResponse>>(response).await;
        assert_eq!(0, result.data.len());
    }

    #[actix_web::test]
    async fn should_return_user_reviews() {
        let (app, pool) = setup::create_app(find_all).await;
        let (user, token, review, movie_review) = {
            let mut conn = pool.get().unwrap();
            let (token, user) = data::create_authed_user(&mut conn);

            let review = Review::create(&mut conn, data::generate_review(user.user_id)).unwrap();
            let movie_review = MovieReview::create(
                &mut conn,
                data::generate_movie_review(user.user_id, review.review_id),
            )
            .unwrap();

            (user, token, review, movie_review)
        };

        let request = test::TestRequest::get()
            .uri("/movies/reviews")
            .insert_header((AUTHORIZATION, format!("Bearer {token}")))
            .to_request();

        let response = test::call_service(&app, request).await;
        assert!(response.status().is_success());

        let result = process::parse_body::<Vec<MovieReviewResponse>>(response).await;
        assert_eq!(1, result.data.len());

        let returned_review = result.data.first().unwrap();
        assert_eq!(review.review_id, returned_review.review_id);
        assert_eq!(review.user_id, user.user_id);
        assert_eq!(review.date, returned_review.date);
        assert_eq!(review.description, returned_review.description);
        assert_eq!(review.rating, returned_review.rating);
        assert_eq!(review.title, returned_review.title);
        assert_eq!(review.venue, returned_review.venue);
        assert_eq!(movie_review.movie_id, returned_review.movie.id);
        assert_eq!(movie_review.title, returned_review.movie.title);
        assert_eq!(movie_review.imdb_id, returned_review.movie.imdb_id);
        assert_eq!(movie_review.poster_path, returned_review.movie.poster_path);
        assert_eq!(
            movie_review.release_date,
            returned_review.movie.release_date
        );
    }
}

mod find_by_review_id {
    use crate::common::{data, process, setup};
    use actix_web::{http::header::AUTHORIZATION, test};
    use framerate::{
        movie_review::{find_by_review_id, MovieReview, MovieReviewResponse},
        review::Review,
    };

    #[actix_web::test]
    async fn should_require_authentication() {
        let (app, pool) = setup::create_app(find_by_review_id).await;
        let review = {
            let mut conn = pool.get().unwrap();
            let user = data::create_user(&mut conn);

            let review = Review::create(&mut conn, data::generate_review(user.user_id)).unwrap();
            MovieReview::create(
                &mut conn,
                data::generate_movie_review(user.user_id, review.review_id),
            )
            .unwrap();

            review
        };

        let request = test::TestRequest::get()
            .uri(&format!("/movies/reviews/{}", review.review_id))
            .to_request();

        let response = test::call_service(&app, request).await;
        assert!(response.status().is_client_error());
    }

    #[actix_web::test]
    async fn should_not_return_other_users_review() {
        let (app, pool) = setup::create_app(find_by_review_id).await;
        let (token, review) = {
            let mut conn = pool.get().unwrap();
            let user = data::create_user(&mut conn);

            let review = Review::create(&mut conn, data::generate_review(user.user_id)).unwrap();
            MovieReview::create(
                &mut conn,
                data::generate_movie_review(user.user_id, review.review_id),
            )
            .unwrap();

            let (token, _) = data::create_authed_user(&mut conn);

            (token, review)
        };

        let request = test::TestRequest::get()
            .uri(&format!("/movies/reviews/{}", review.review_id))
            .insert_header((AUTHORIZATION, format!("Bearer {token}")))
            .to_request();

        let response = test::call_service(&app, request).await;
        assert!(response.status().is_client_error());
    }

    #[actix_web::test]
    async fn should_return_review_details() {
        let (app, pool) = setup::create_app(find_by_review_id).await;
        let (user, token, review, movie_review) = {
            let mut conn = pool.get().unwrap();
            let (token, user) = data::create_authed_user(&mut conn);

            let review = Review::create(&mut conn, data::generate_review(user.user_id)).unwrap();
            let movie_review = MovieReview::create(
                &mut conn,
                data::generate_movie_review(user.user_id, review.review_id),
            )
            .unwrap();

            (user, token, review, movie_review)
        };

        let request = test::TestRequest::get()
            .uri(&format!("/movies/reviews/{}", review.review_id))
            .insert_header((AUTHORIZATION, format!("Bearer {token}")))
            .to_request();

        let response = test::call_service(&app, request).await;
        assert!(response.status().is_success());

        let result = process::parse_body::<MovieReviewResponse>(response).await;

        assert_eq!(review.review_id, result.data.review_id);
        assert_eq!(review.user_id, user.user_id);
        assert_eq!(review.date, result.data.date);
        assert_eq!(review.description, result.data.description);
        assert_eq!(review.rating, result.data.rating);
        assert_eq!(review.title, result.data.title);
        assert_eq!(review.venue, result.data.venue);
        assert_eq!(movie_review.movie_id, result.data.movie.id);
        assert_eq!(movie_review.title, result.data.movie.title);
        assert_eq!(movie_review.imdb_id, result.data.movie.imdb_id);
        assert_eq!(movie_review.poster_path, result.data.movie.poster_path);
        assert_eq!(movie_review.release_date, result.data.movie.release_date);
    }
}

mod find_by_movie_id {
    use crate::common::{data, process, setup};
    use actix_web::{http::header::AUTHORIZATION, test};
    use framerate::{
        movie_review::{find_by_movie_id, MovieReview, MovieReviewResponse},
        review::Review,
    };

    #[actix_web::test]
    async fn should_require_authentication() {
        let (app, pool) = setup::create_app(find_by_movie_id).await;
        let movie_review = {
            let mut conn = pool.get().unwrap();
            let user = data::create_user(&mut conn);

            let review = Review::create(&mut conn, data::generate_review(user.user_id)).unwrap();
            let movie_review = MovieReview::create(
                &mut conn,
                data::generate_movie_review(user.user_id, review.review_id),
            )
            .unwrap();

            movie_review
        };

        let request = test::TestRequest::get()
            .uri(&format!("/movies/{}/reviews", movie_review.movie_id))
            .to_request();

        let response = test::call_service(&app, request).await;
        assert!(response.status().is_client_error());
    }

    #[actix_web::test]
    async fn should_not_return_other_users_reviews() {
        let (app, pool) = setup::create_app(find_by_movie_id).await;
        let (token, movie_review) = {
            let mut conn = pool.get().unwrap();
            let user = data::create_user(&mut conn);

            let review = Review::create(&mut conn, data::generate_review(user.user_id)).unwrap();
            let movie_review = MovieReview::create(
                &mut conn,
                data::generate_movie_review(user.user_id, review.review_id),
            )
            .unwrap();

            let (token, _) = data::create_authed_user(&mut conn);

            (token, movie_review)
        };

        let request = test::TestRequest::get()
            .uri(&format!("/movies/{}/reviews", movie_review.movie_id))
            .insert_header((AUTHORIZATION, format!("Bearer {token}")))
            .to_request();

        let response = test::call_service(&app, request).await;
        assert!(response.status().is_success());

        let result = process::parse_body::<Vec<MovieReviewResponse>>(response).await;
        assert_eq!(0, result.data.len());
    }

    #[actix_web::test]
    async fn should_return_review_details() {
        let (app, pool) = setup::create_app(find_by_movie_id).await;
        let (user, token, review, movie_review) = {
            let mut conn = pool.get().unwrap();
            let (token, user) = data::create_authed_user(&mut conn);

            let review = Review::create(&mut conn, data::generate_review(user.user_id)).unwrap();
            let movie_review = MovieReview::create(
                &mut conn,
                data::generate_movie_review(user.user_id, review.review_id),
            )
            .unwrap();

            (user, token, review, movie_review)
        };

        let request = test::TestRequest::get()
            .uri(&format!("/movies/{}/reviews", movie_review.movie_id))
            .insert_header((AUTHORIZATION, format!("Bearer {token}")))
            .to_request();

        let response = test::call_service(&app, request).await;
        assert!(response.status().is_success());

        let result = process::parse_body::<Vec<MovieReviewResponse>>(response).await;
        assert_eq!(1, result.data.len());

        let returned_review = result.data.first().unwrap();

        assert_eq!(review.review_id, returned_review.review_id);
        assert_eq!(review.user_id, user.user_id);
        assert_eq!(review.date, returned_review.date);
        assert_eq!(review.description, returned_review.description);
        assert_eq!(review.rating, returned_review.rating);
        assert_eq!(review.title, returned_review.title);
        assert_eq!(review.venue, returned_review.venue);
        assert_eq!(movie_review.movie_id, returned_review.movie.id);
        assert_eq!(movie_review.title, returned_review.movie.title);
        assert_eq!(movie_review.imdb_id, returned_review.movie.imdb_id);
        assert_eq!(movie_review.poster_path, returned_review.movie.poster_path);
        assert_eq!(
            movie_review.release_date,
            returned_review.movie.release_date
        );
    }
}

mod create {
    use crate::common::{data, process, setup};
    use actix_web::{http::header::AUTHORIZATION, test};
    use chrono::Utc;
    use framerate::{
        movie_review::{create, MovieReview, MovieReviewResponse, SaveMovieReviewRequest},
        review_company::ReviewCompanySummary,
    };
    use rand::{self, Rng};
    use uuid::Uuid;

    #[actix_web::test]
    async fn should_require_authentication() {
        let (app, _) = setup::create_app(create).await;

        let request = test::TestRequest::post()
            .uri(&format!("/movies/reviews"))
            .to_request();

        let response = test::call_service(&app, request).await;
        assert!(response.status().is_client_error());
    }

    #[actix_web::test]
    async fn should_discard_review_on_company_save_error() {
        let (app, pool) = setup::create_app(create).await;
        let (token, user) = {
            let mut conn = pool.get().unwrap();
            let (token, user) = data::create_authed_user(&mut conn);
            (token, user)
        };

        let movie = data::generate_sample_movie();
        let review = data::generate_save_movie_review().company(vec![ReviewCompanySummary {
            user_id: Uuid::new_v4(),
        }]);

        let request = test::TestRequest::post()
            .uri(&format!("/movies/{}/reviews", movie.id))
            .set_json(&review)
            .insert_header((AUTHORIZATION, format!("Bearer {token}")))
            .to_request();

        let response = test::call_service(&app, request).await;
        assert!(response.status().is_client_error());

        let reviews = {
            let mut conn = pool.get().unwrap();
            MovieReview::find_by_movie_id(&mut conn, user.user_id, movie.id).unwrap()
        };

        assert_eq!(0, reviews.len());
    }

    #[actix_web::test]
    async fn should_create_review() {
        let (app, pool) = setup::create_app(create).await;
        let (token, user) = {
            let mut conn = pool.get().unwrap();
            let (token, user) = data::create_authed_user(&mut conn);
            (token, user)
        };

        let movie = data::generate_sample_movie();
        let review = data::generate_save_movie_review();

        let request = test::TestRequest::post()
            .uri(&format!("/movies/{}/reviews", movie.id))
            .set_json(&review)
            .insert_header((AUTHORIZATION, format!("Bearer {token}")))
            .to_request();

        let response = test::call_service(&app, request).await;
        assert!(response.status().is_success());

        let result = process::parse_body::<MovieReviewResponse>(response).await;

        assert!(!result.data.review_id.is_nil());
        assert_eq!(user.user_id, result.data.user_id);
        assert_eq!(review.date, result.data.date);
        assert_eq!(review.description, result.data.description);
        assert_eq!(review.rating, result.data.rating);
        assert_eq!(review.title, result.data.title);
        assert_eq!(review.venue, result.data.venue);
        assert_eq!(movie.id, result.data.movie.id);
        assert_eq!(movie.imdb_id, result.data.movie.imdb_id);
        assert_eq!(movie.poster_path, result.data.movie.poster_path);
        assert_eq!(movie.release_date, result.data.movie.release_date);
    }

    #[actix_web::test]
    async fn should_create_review_with_company() {
        let (app, pool) = setup::create_app(create).await;
        let (token, company_details) = {
            let mut conn = pool.get().unwrap();
            let (token, _) = data::create_authed_user(&mut conn);
            let company_details = data::create_user(&mut conn);
            (token, company_details)
        };

        let mut rng = rand::thread_rng();

        let movie = data::generate_sample_movie();

        let review = SaveMovieReviewRequest {
            title: Some(Uuid::new_v4().to_string()),
            date: Some(Utc::now().naive_utc().date()),
            rating: rng.gen(),
            description: Some(Uuid::new_v4().to_string()),
            venue: Some(Uuid::new_v4().to_string()),
            company: Some(vec![ReviewCompanySummary {
                user_id: company_details.user_id,
            }]),
        };

        let request = test::TestRequest::post()
            .uri(&format!("/movies/{}/reviews", movie.id))
            .set_json(&review)
            .insert_header((AUTHORIZATION, format!("Bearer {token}")))
            .to_request();

        let response = test::call_service(&app, request).await;
        assert!(response.status().is_success());

        let result = process::parse_body::<MovieReviewResponse>(response).await;

        assert!(!result.data.review_id.is_nil());

        let review_company = review.company.unwrap();
        let result_company = result.data.company.unwrap();

        assert_eq!(review_company.len(), result_company.len());

        let review_company = review_company.first().unwrap();
        let result_company = result_company.first().unwrap();

        assert_eq!(review_company.user_id, result_company.user_id);
        assert_eq!(company_details.first_name, result_company.first_name);
        assert_eq!(company_details.last_name, result_company.last_name);
    }
}
