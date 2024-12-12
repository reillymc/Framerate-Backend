pub mod common;

mod details {
    use crate::common::{data, process, setup};
    use actix_web::{http::header::AUTHORIZATION, test};
    use framerate::season::{details, Season};

    #[actix_web::test]
    async fn should_require_authentication() {
        let (app, _) = setup::create_app(details).await;

        let show = data::generate_sample_show();

        let request = test::TestRequest::get()
            .uri(&format!("/shows/{}/seasons/{}/details", show.id, 1))
            .to_request();

        let response = test::call_service(&app, request).await;
        assert_eq!(401, response.status());
    }

    #[actix_web::test]
    async fn should_return_season() {
        let (app, pool) = setup::create_app(details).await;
        let (token, _) = {
            let mut conn = pool.get().unwrap();
            data::create_authed_user(&mut conn)
        };

        let show = data::generate_sample_show();

        let request = test::TestRequest::get()
            .uri(&format!("/shows/{}/seasons/{}/details", show.id, 1))
            .insert_header((AUTHORIZATION, format!("Bearer {token}")))
            .to_request();

        let response = test::call_service(&app, request).await;
        assert!(response.status().is_success());

        let result = process::parse_body::<Season>(response).await;
        assert_eq!(show.id, result.data.show_id);
        assert_eq!(1, result.data.season_number);
    }
}
