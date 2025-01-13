use actix_web::{body::BoxBody, http::header::ContentType, HttpRequest, HttpResponse, Responder};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

#[derive(Serialize, Deserialize)]
pub struct Success<T> {
    pub data: T,
}

impl<T: Serialize> Success<T> {
    pub fn new(data: T) -> Success<T> {
        Success { data }
    }

    // pub fn message(mut self, message: &str) -> Self {
    //     self.message = Some(message.to_string());
    //     self
    // }
}

#[derive(Serialize)]
pub struct Error {
    pub message: String,
}

impl<T: Serialize> Responder for Success<T> {
    type Body = BoxBody;

    fn respond_to(self, _req: &HttpRequest) -> HttpResponse<Self::Body> {
        let body = serde_json::to_string(&self.data).unwrap();

        HttpResponse::Ok()
            .content_type(ContentType::json())
            .body(body)
    }
}

#[derive(Deserialize, Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct DeleteResponse {
    pub count: usize,
}
