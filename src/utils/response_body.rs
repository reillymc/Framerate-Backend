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
