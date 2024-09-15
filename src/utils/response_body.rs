use serde::Serialize;

#[derive(Serialize)]
pub struct Success<T> {
    pub data: T,
}

#[derive(Serialize)]
pub struct Error {
    pub message: String,
}
