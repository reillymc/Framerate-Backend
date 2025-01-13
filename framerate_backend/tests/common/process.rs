use actix_web::dev::ServiceResponse;
use actix_web::test;
use serde::Deserialize;

#[derive(Deserialize)]
pub struct Success<T> {
    pub data: T,
}

impl<T> Success<T> {
    pub fn new(data: T) -> Success<T> {
        Success { data }
    }
}

pub async fn parse_body<T: for<'a> Deserialize<'a>>(response: ServiceResponse) -> Success<T> {
    let body = test::read_body(response).await;
    // println!("{:?}", body);
    let data = serde_json::from_slice(&body).unwrap();

    Success::new(data)
}
