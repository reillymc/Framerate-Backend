pub mod common;

mod find_all {
    use crate::common::{data, process, setup};
    use actix_web::{http::header::AUTHORIZATION, test};
    use framerate::{movie_review::find_all, movie_review::MovieReviewResponse};

    #[actix_web::test]
    async fn should_require_authentication() {
        let (app, _) = setup::create_app(find_all).await;

        let request = test::TestRequest::get().uri("/movies/reviews").to_request();

        let response = test::call_service(&app, request).await;
        assert_eq!(401, response.status());
    }

    #[actix_web::test]
    async fn should_not_return_other_users_reviews() {
        let (app, pool) = setup::create_app(find_all).await;

        let token = {
            let mut conn = pool.get().unwrap();
            let user = data::create_user(&mut conn);
            let review = data::create_review(&mut conn, &user);
            data::create_movie_review(&mut conn, &user, &review);
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
            let review = data::create_review(&mut conn, &user);
            let movie_review = data::create_movie_review(&mut conn, &user, &review);
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
        assert_eq!(user.user_id, returned_review.user_id);
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
    use framerate::movie_review::{find_by_review_id, MovieReviewResponse};

    #[actix_web::test]
    async fn should_require_authentication() {
        let (app, pool) = setup::create_app(find_by_review_id).await;

        let movie_review = {
            let mut conn = pool.get().unwrap();
            let user = data::create_user(&mut conn);
            let review = data::create_review(&mut conn, &user);
            data::create_movie_review(&mut conn, &user, &review)
        };

        let request = test::TestRequest::get()
            .uri(&format!("/movies/reviews/{}", movie_review.review_id))
            .to_request();

        let response = test::call_service(&app, request).await;
        assert_eq!(401, response.status());
    }

    #[actix_web::test]
    async fn should_not_return_other_users_review() {
        let (app, pool) = setup::create_app(find_by_review_id).await;

        let (token, movie_review) = {
            let mut conn = pool.get().unwrap();
            let user = data::create_user(&mut conn);
            let review = data::create_review(&mut conn, &user);
            let movie_review = data::create_movie_review(&mut conn, &user, &review);
            let (token, _) = data::create_authed_user(&mut conn);
            (token, movie_review)
        };

        let request = test::TestRequest::get()
            .uri(&format!("/movies/reviews/{}", movie_review.review_id))
            .insert_header((AUTHORIZATION, format!("Bearer {token}")))
            .to_request();

        let response = test::call_service(&app, request).await;
        assert_eq!(404, response.status());
    }

    #[actix_web::test]
    async fn should_return_review_details() {
        let (app, pool) = setup::create_app(find_by_review_id).await;

        let (user, token, review, movie_review) = {
            let mut conn = pool.get().unwrap();
            let (token, user) = data::create_authed_user(&mut conn);
            let review = data::create_review(&mut conn, &user);
            let movie_review = data::create_movie_review(&mut conn, &user, &review);
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
        assert_eq!(user.user_id, result.data.user_id);
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
    use framerate::movie_review::{find_by_movie_id, MovieReviewResponse};

    #[actix_web::test]
    async fn should_require_authentication() {
        let (app, pool) = setup::create_app(find_by_movie_id).await;

        let movie_review = {
            let mut conn = pool.get().unwrap();
            let user = data::create_user(&mut conn);

            let review = data::create_review(&mut conn, &user);
            let movie_review = data::create_movie_review(&mut conn, &user, &review);

            movie_review
        };

        let request = test::TestRequest::get()
            .uri(&format!("/movies/{}/reviews", movie_review.movie_id))
            .to_request();

        let response = test::call_service(&app, request).await;
        assert_eq!(401, response.status());
    }

    #[actix_web::test]
    async fn should_not_return_other_users_reviews() {
        let (app, pool) = setup::create_app(find_by_movie_id).await;

        let (token, movie_review) = {
            let mut conn = pool.get().unwrap();
            let user = data::create_user(&mut conn);
            let review = data::create_review(&mut conn, &user);
            let movie_review = data::create_movie_review(&mut conn, &user, &review);
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
            let review = data::create_review(&mut conn, &user);
            let movie_review = data::create_movie_review(&mut conn, &user, &review);
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
        assert_eq!(user.user_id, returned_review.user_id);
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
    use framerate::{
        movie_review::{create, MovieReview, MovieReviewResponse},
        review_company::ReviewCompanySummary,
    };
    use uuid::Uuid;

    #[actix_web::test]
    async fn should_require_authentication() {
        let (app, _) = setup::create_app(create).await;

        let movie = data::generate_sample_movie();

        let request = test::TestRequest::post()
            .uri(&format!("/movies/{}/reviews", movie.id))
            .to_request();

        let response = test::call_service(&app, request).await;
        assert_eq!(401, response.status());
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
            company_id: Uuid::new_v4(),
        }]);

        let request = test::TestRequest::post()
            .uri(&format!("/movies/{}/reviews", movie.id))
            .set_json(&review)
            .insert_header((AUTHORIZATION, format!("Bearer {token}")))
            .to_request();

        let response = test::call_service(&app, request).await;
        assert!(response.status().is_server_error());

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
        assert!(result.data.movie.poster_path.is_some());
        assert_eq!(movie.release_date, result.data.movie.release_date);
    }

    #[actix_web::test]
    async fn should_create_review_with_company() {
        let (app, pool) = setup::create_app(create).await;

        let (token, company_details) = {
            let mut conn = pool.get().unwrap();
            let (token, user) = data::create_authed_user(&mut conn);
            let company_details = data::create_company(&mut conn, &user);
            (token, company_details)
        };

        let movie = data::generate_sample_movie();

        let review = data::generate_save_movie_review().company(vec![ReviewCompanySummary {
            company_id: company_details.company_id,
        }]);

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

        assert_eq!(review_company.company_id, result_company.company_id);
        assert_eq!(company_details.first_name, result_company.first_name);
        assert_eq!(company_details.last_name, result_company.last_name);
    }
}

mod update {
    use crate::common::{data, process, setup};
    use actix_web::{http::header::AUTHORIZATION, test};
    use framerate::{
        movie_review::{update, MovieReview, MovieReviewResponse},
        review_company::{ReviewCompany, ReviewCompanySummary},
    };
    use uuid::Uuid;

    #[actix_web::test]
    async fn should_require_authentication() {
        let (app, pool) = setup::create_app(update).await;

        let movie_review = {
            let mut conn = pool.get().unwrap();
            let user = data::create_user(&mut conn);
            let review = data::create_review(&mut conn, &user);
            data::create_movie_review(&mut conn, &user, &review)
        };

        let updated_review = data::generate_save_movie_review();

        let request = test::TestRequest::put()
            .uri(&format!(
                "/movies/{}/reviews/{}",
                movie_review.movie_id, movie_review.review_id
            ))
            .set_json(&updated_review)
            .to_request();

        let response = test::call_service(&app, request).await;
        assert_eq!(401, response.status());
    }

    #[actix_web::test]
    async fn should_not_update_other_users_review() {
        let (app, pool) = setup::create_app(update).await;

        let (token, movie_review) = {
            let mut conn = pool.get().unwrap();
            let user = data::create_user(&mut conn);
            let review = data::create_review(&mut conn, &user);
            let movie_review = data::create_movie_review(&mut conn, &user, &review);
            let (token, _) = data::create_authed_user(&mut conn);
            (token, movie_review)
        };

        let updated_review = data::generate_save_movie_review();

        let request = test::TestRequest::put()
            .uri(&format!(
                "/movies/{}/reviews/{}",
                movie_review.movie_id, movie_review.review_id
            ))
            .insert_header((AUTHORIZATION, format!("Bearer {token}")))
            .set_json(&updated_review)
            .to_request();

        let response = test::call_service(&app, request).await;
        assert_eq!(404, response.status());
    }

    #[actix_web::test]
    async fn should_discard_review_changes_on_company_save_error() {
        let (app, pool) = setup::create_app(update).await;

        let (token, user, review, movie_review) = {
            let mut conn = pool.get().unwrap();
            let (token, user) = data::create_authed_user(&mut conn);
            let review = data::create_review(&mut conn, &user);
            let movie_review = data::create_movie_review(&mut conn, &user, &review);
            (token, user, review, movie_review)
        };

        let updated_review =
            data::generate_save_movie_review().company(vec![ReviewCompanySummary {
                company_id: Uuid::new_v4(),
            }]);

        let request = test::TestRequest::put()
            .uri(&format!(
                "/movies/{}/reviews/{}",
                movie_review.movie_id, movie_review.review_id
            ))
            .set_json(&updated_review)
            .insert_header((AUTHORIZATION, format!("Bearer {token}")))
            .to_request();

        let response = test::call_service(&app, request).await;
        assert!(response.status().is_server_error());

        let (review_response, company_response) = {
            let mut conn = pool.get().unwrap();
            let review =
                MovieReview::find_by_review_id(&mut conn, user.user_id, movie_review.review_id)
                    .unwrap();
            let company = ReviewCompany::find_by_review(&mut conn, movie_review.review_id).unwrap();
            (review, company)
        };

        assert_eq!(review.date, review_response.date);
        assert_eq!(review.description, review_response.description);
        assert_eq!(review.rating, review_response.rating);
        assert_eq!(review.review_id, review_response.review_id);
        assert_eq!(review.title, review_response.title);
        assert_eq!(review.user_id, review_response.user_id);
        assert_eq!(review.venue, review_response.venue);
        assert_eq!(0, company_response.len());
    }

    #[actix_web::test]
    async fn should_update_review() {
        let (app, pool) = setup::create_app(update).await;

        let (token, review, movie_review) = {
            let mut conn = pool.get().unwrap();
            let (token, user) = data::create_authed_user(&mut conn);
            let review = data::create_review(&mut conn, &user);
            let movie_review = data::create_movie_review(&mut conn, &user, &review);
            (token, review, movie_review)
        };

        let updated_review = data::generate_save_movie_review();

        let request = test::TestRequest::put()
            .uri(&format!(
                "/movies/{}/reviews/{}",
                movie_review.movie_id, movie_review.review_id
            ))
            .set_json(&updated_review)
            .insert_header((AUTHORIZATION, format!("Bearer {token}")))
            .to_request();

        let response = test::call_service(&app, request).await;
        assert!(response.status().is_success());

        let review_response = process::parse_body::<MovieReviewResponse>(response)
            .await
            .data;

        assert_eq!(review.review_id, review_response.review_id);
        assert_eq!(review.user_id, review_response.user_id);
        assert_eq!(updated_review.date, review_response.date);
        assert_eq!(updated_review.description, review_response.description);
        assert_eq!(updated_review.rating, review_response.rating);
        assert_eq!(updated_review.title, review_response.title);
        assert_eq!(updated_review.venue, review_response.venue);
    }

    #[actix_web::test]
    async fn should_update_review_company() {
        let (app, pool) = setup::create_app(update).await;

        let (token, movie_review, company_user1, company_user2) = {
            let mut conn = pool.get().unwrap();
            let (token, user) = data::create_authed_user(&mut conn);
            let review = data::create_review(&mut conn, &user);
            let movie_review = data::create_movie_review(&mut conn, &user, &review);
            let company_user1 = data::create_company(&mut conn, &user);
            let company_user2 = data::create_company(&mut conn, &user);

            ReviewCompany::replace(
                &mut conn,
                movie_review.review_id,
                Some(&vec![ReviewCompanySummary {
                    company_id: company_user1.company_id,
                }]),
            )
            .unwrap();

            (token, movie_review, company_user1, company_user2)
        };

        // Add a user to review company
        let updated_review = data::generate_save_movie_review().company(vec![
            ReviewCompanySummary {
                company_id: company_user1.company_id,
            },
            ReviewCompanySummary {
                company_id: company_user2.company_id,
            },
        ]);

        let request = test::TestRequest::put()
            .uri(&format!(
                "/movies/{}/reviews/{}",
                movie_review.movie_id, movie_review.review_id
            ))
            .set_json(&updated_review)
            .insert_header((AUTHORIZATION, format!("Bearer {token}")))
            .to_request();

        let response = test::call_service(&app, request).await;
        assert!(response.status().is_success());
        let review_response = process::parse_body::<MovieReviewResponse>(response)
            .await
            .data;

        let company = review_response.company.unwrap();

        assert_eq!(&2, &company.len());
        assert!(&company
            .iter()
            .find(|company| company.company_id == company_user1.company_id)
            .is_some());
        assert!(&company
            .iter()
            .find(|company| company.company_id == company_user2.company_id)
            .is_some());

        // Remove a user from review company
        let updated_review =
            data::generate_save_movie_review().company(vec![ReviewCompanySummary {
                company_id: company_user2.company_id,
            }]);

        let request = test::TestRequest::put()
            .uri(&format!(
                "/movies/{}/reviews/{}",
                movie_review.movie_id, movie_review.review_id
            ))
            .set_json(&updated_review)
            .insert_header((AUTHORIZATION, format!("Bearer {token}")))
            .to_request();

        let response = test::call_service(&app, request).await;
        assert!(response.status().is_success());
        let review_response = process::parse_body::<MovieReviewResponse>(response)
            .await
            .data;

        let company = review_response.company.unwrap();

        assert_eq!(&1, &company.len());
        assert!(&company
            .iter()
            .find(|company| company.company_id == company_user2.company_id)
            .is_some());

        // Clear review company
        let updated_review = data::generate_save_movie_review();

        let request = test::TestRequest::put()
            .uri(&format!(
                "/movies/{}/reviews/{}",
                movie_review.movie_id, movie_review.review_id
            ))
            .set_json(&updated_review)
            .insert_header((AUTHORIZATION, format!("Bearer {token}")))
            .to_request();

        let response = test::call_service(&app, request).await;
        assert!(response.status().is_success());
        let review_response = process::parse_body::<MovieReviewResponse>(response)
            .await
            .data;

        let company = review_response.company.unwrap();

        assert_eq!(&0, &company.len());
    }
}
