use crate::db::DbPool;
use crate::user::NewUser;
use crate::utils::response_body::DeleteResponse;
use crate::utils::{jwt::Auth, response_body::Success, AppError};
use actix_web::{delete, get, post, put, web, Responder};
use serde::Serialize;
use utoipa::ToSchema;
use uuid::Uuid;

use super::{UpdatedUser, User, UserFindResponse, UserResponse};

#[derive(Serialize, ToSchema)]
#[serde(untagged)]
enum UserRead {
    Type1(UserFindResponse),
    Type2(UserResponse),
}

#[utoipa::path(tag = "User", responses((status = OK, body = Vec<User>)))]
#[get("/users")]
async fn find_all(pool: web::Data<DbPool>, auth: Auth) -> actix_web::Result<impl Responder> {
    if !auth.is_at_least_admin() {
        return Err(AppError::external(401, "Unauthorized to list users"))?;
    }

    let users = web::block(move || {
        let mut conn = pool.get()?;
        User::find_all(&mut conn)
    })
    .await??;

    Ok(Success::new(users))
}

#[utoipa::path(tag = "User", responses((status = OK, body = UserRead)))]
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
        Ok::<UserRead, AppError>(user)
    })
    .await??;

    Ok(Success::new(user))
}

#[utoipa::path(tag = "User", responses((status = OK, body = User)))]
#[post("/users")]
async fn create(
    pool: web::Data<DbPool>,
    auth: Auth,
    user: web::Json<NewUser>,
) -> actix_web::Result<impl Responder> {
    if !auth.is_at_least_admin() {
        return Err(AppError::external(401, "Unauthorized to create users"))?;
    }

    if user.email.is_empty() || user.password.is_empty() {
        return Err(AppError::external(400, "Email and password are mandatory"))?;
    }

    let user = web::block(move || {
        let mut conn = pool.get()?;
        User::create(&mut conn, user.into_inner())
    })
    .await??;

    Ok(Success::new(user))
}

#[utoipa::path(tag = "User", responses((status = OK, body = User)))]
#[put("/users/{user_id}")]
async fn update(
    pool: web::Data<DbPool>,
    auth: Auth,
    user_id: web::Path<Uuid>,
    user: web::Json<UpdatedUser>,
) -> actix_web::Result<impl Responder> {
    if auth.user_id != *user_id {
        return Err(AppError::external(404, "User not found"))?;
    }

    let user = web::block(move || {
        let mut conn = pool.get()?;
        User::update(&mut conn, user_id.into_inner(), user.into_inner())
    })
    .await??;

    Ok(Success::new(user))
}

#[utoipa::path(tag = "User", responses((status = OK, body = DeleteResponse)))]
#[delete("/users/{user_id}")]
async fn delete(
    pool: web::Data<DbPool>,
    auth: Auth,
    user_id: web::Path<Uuid>,
) -> actix_web::Result<impl Responder> {
    if auth.user_id != *user_id {
        return Err(AppError::external(404, "User not found"))?;
    }

    let count = web::block(move || {
        let mut conn = pool.get()?;
        User::delete(&mut conn, user_id.into_inner())
    })
    .await??;

    Ok(Success::new(DeleteResponse { count }))
}
