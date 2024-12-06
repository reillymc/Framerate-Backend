mod common;

use crate::common::{data, process, setup};
use actix_web::{http::header::AUTHORIZATION, test};
use framerate::show::{search, Show};

#[actix_web::test]
async fn search_should_return_results() {
    let (app, mut conn) = setup::create_app(search).await;
    let (_, token) = data::get_authed_user(&mut conn);

    let request = test::TestRequest::get()
        .uri("/shows/search?query=Doctor%20Who")
        .insert_header((AUTHORIZATION, format!("Bearer {token}")))
        .to_request();

    let response = test::call_service(&app, request).await;
    assert!(response.status().is_success());

    let result = process::parse_body::<Vec<Show>>(response).await;
    assert!(result.data.len() > 0);
}

#[actix_web::test]
async fn search_should_require_authentication() {
    let (app, _) = setup::create_app(search).await;

    let request = test::TestRequest::get()
        .uri("/shows/search?query=Doctor%20Who")
        .to_request();

    let response = test::call_service(&app, request).await;
    assert!(response.status().is_client_error());
}
