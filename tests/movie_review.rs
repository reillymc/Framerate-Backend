mod common;

mod search {
    use crate::common::{data, process, setup};
    use actix_web::{http::header::AUTHORIZATION, test};
    use chrono::Utc;
    use framerate::{
        movie_review::find_all,
        movie_review::{MovieReview, MovieReviewResponse},
        review::Review,
    };
    use rand::{self, Rng};
    use uuid::Uuid;

    #[actix_web::test]
    async fn should_require_authentication() {
        let (app, _) = setup::create_app(find_all).await;

        let request = test::TestRequest::get().uri("/movies/reviews").to_request();

        let response = test::call_service(&app, request).await;
        assert!(response.status().is_client_error());
    }

    #[actix_web::test]
    async fn should_return_user_reviews() {
        let (app, pool) = setup::create_app(find_all).await;
        let (user, token, review, movie_review) = {
            let mut conn = pool.get().unwrap();
            let (user, token) = data::get_authed_user(&mut conn);
            let mut rng = rand::thread_rng();

            let review = Review::create(
                &mut conn,
                Review {
                    review_id: Uuid::new_v4(),
                    user_id: user.user_id,
                    title: Some(Uuid::new_v4().to_string()),
                    date: Some(Utc::now().naive_utc().date()),
                    rating: rng.gen(),
                    description: Some(Uuid::new_v4().to_string()),
                    venue: Some(Uuid::new_v4().to_string()),
                },
            )
            .unwrap();

            let movie_review = MovieReview::create(
                &mut conn,
                MovieReview {
                    review_id: review.review_id,
                    imdb_id: Some(Uuid::new_v4().to_string()),
                    user_id: user.user_id,
                    movie_id: rng.gen(),
                    title: Uuid::new_v4().to_string(),
                    poster_path: Some(Uuid::new_v4().to_string()),
                    release_date: Some(Utc::now().naive_utc().date()),
                },
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

    #[actix_web::test]
    async fn should_not_return_other_users_reviews() {
        let (app, pool) = setup::create_app(find_all).await;
        let token = {
            let mut conn = pool.get().unwrap();
            let (user, _) = data::get_authed_user(&mut conn);
            let mut rng = rand::thread_rng();

            let review = Review::create(
                &mut conn,
                Review {
                    review_id: Uuid::new_v4(),
                    user_id: user.user_id,
                    title: Some(Uuid::new_v4().to_string()),
                    date: Some(Utc::now().naive_utc().date()),
                    rating: rng.gen(),
                    description: Some(Uuid::new_v4().to_string()),
                    venue: Some(Uuid::new_v4().to_string()),
                },
            )
            .unwrap();

            MovieReview::create(
                &mut conn,
                MovieReview {
                    review_id: review.review_id,
                    imdb_id: Some(Uuid::new_v4().to_string()),
                    user_id: user.user_id,
                    movie_id: rng.gen(),
                    title: Uuid::new_v4().to_string(),
                    poster_path: Some(Uuid::new_v4().to_string()),
                    release_date: Some(Utc::now().naive_utc().date()),
                },
            )
            .unwrap();

            let (_, token) = data::get_authed_user(&mut conn);

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
}
