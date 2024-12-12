use actix_web::dev::ServiceResponse;
use actix_web::test;
use framerate::utils::response_body::Success;
use serde::Deserialize;

pub async fn parse_body<T: for<'a> Deserialize<'a>>(response: ServiceResponse) -> Success<T> {
    let body = test::read_body(response).await;
    // println!("{:?}", body);
    serde_json::from_slice(&body).unwrap()
}
