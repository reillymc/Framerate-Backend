use crate::{
    db::DbPool,
    error_handler::CustomError,
    user::NewUser,
    utils::{jwt::Auth, response_body::Success},
};
use actix_web::{get, post, put, web, Responder};
use serde::Serialize;
use uuid::Uuid;

use super::{UpdatedUser, User, UserFindResponse, UserResponse};

#[derive(Serialize)]
#[serde(untagged)]
enum UserRead {
    Type1(UserFindResponse),
    Type2(UserResponse),
}

#[get("/users")]
async fn find_all(pool: web::Data<DbPool>, auth: Auth) -> actix_web::Result<impl Responder> {
    if !auth.is_at_least_admin() {
        return Err(CustomError::new(401, "Unauthorized to list users"))?;
    }

    let users = web::block(move || {
        let mut conn = pool.get()?;
        User::find_all(&mut conn)
    })
    .await??;

    Ok(Success::new(users))
}

#[get("/users/{user_id}")]
async fn find(
    pool: web::Data<DbPool>,
    auth: Auth,
    user_id: web::Path<Uuid>,
) -> actix_web::Result<impl Responder> {
    let user = web::block(move || {
        let mut conn = pool.get()?;
        let user = if auth.user_id == *user_id {
            UserRead::Type1(User::find(&mut conn, user_id.into_inner())?)
        } else {
            UserRead::Type2(User::find_summary(&mut conn, user_id.into_inner())?)
        };
        Ok::<UserRead, CustomError>(user)
    })
    .await??;

    Ok(Success::new(user))
}

#[post("/users")]
async fn create(
    pool: web::Data<DbPool>,
    auth: Auth,
    user: web::Json<NewUser>,
) -> actix_web::Result<impl Responder> {
    if !auth.is_at_least_admin() {
        return Err(CustomError::new(401, "Unauthorized to create users"))?;
    }

    if user.email.is_empty() || user.password.is_empty() {
        return Err(CustomError::new(400, "Email and password are mandatory"))?;
    }

    let user = web::block(move || {
        let mut conn = pool.get()?;
        User::create(&mut conn, user.into_inner())
    })
    .await??;

    Ok(Success::new(user))
}

#[put("/users/{user_id}")]
async fn update(
    pool: web::Data<DbPool>,
    auth: Auth,
    user_id: web::Path<Uuid>,
    user: web::Json<UpdatedUser>,
) -> actix_web::Result<impl Responder> {
    if auth.user_id != *user_id {
        return Err(CustomError::new(404, "User not found"))?;
    }

    let user = web::block(move || {
        let mut conn = pool.get()?;
        User::update(&mut conn, user_id.into_inner(), user.into_inner())
    })
    .await??;

    Ok(Success::new(user))
}
