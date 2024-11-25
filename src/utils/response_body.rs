use actix_web::{body::BoxBody, http::header::ContentType, HttpRequest, HttpResponse, Responder};
use serde::Serialize;

#[derive(Serialize)]
pub struct Success<T> {
    pub data: T,
}

#[derive(Serialize)]
pub struct SuccessWithMessage<T> {
    pub data: T,
    pub message: String,
}

#[derive(Serialize)]
pub struct Error {
    pub message: String,
}

impl<T: Serialize> Responder for Success<T> {
    type Body = BoxBody;

    fn respond_to(self, _req: &HttpRequest) -> HttpResponse<Self::Body> {
        let body = serde_json::to_string(&self).unwrap();

        HttpResponse::Ok()
            .content_type(ContentType::json())
            .body(body)
    }
}
