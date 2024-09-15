use crate::{
    user::AuthUser,
    utils::{
        jwt::create_token,
        response_body::{Error, Success},
    },
};
use actix_web::{post, web, HttpResponse, Responder};

#[post("/auth/login")]
pub async fn login(auth_user: web::Json<AuthUser>) -> impl Responder {
    let user = auth_user.login();

    let Ok(user_details) = user else {
        return HttpResponse::BadRequest().json(Error {
            message: "Invalid credentials".to_string(),
        });
    };

    let token = create_token(user_details.user_id, &user_details.email);

    let Ok(token_string) = token else {
        return HttpResponse::InternalServerError().json(Error {
            message: "Unable to create token".to_string(),
        });
    };
    HttpResponse::Ok().json(Success { data: token_string })
}
