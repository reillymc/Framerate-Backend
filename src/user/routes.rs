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
async fn find_all(pool: web::Data<DbPool>, _: Auth) -> actix_web::Result<impl Responder> {
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
        let user = if auth.user_id == user_id.clone() {
            UserRead::Type1(User::find(&mut conn, user_id.into_inner())?)
        } else {
            UserRead::Type2(User::find_summary(&mut conn, user_id.into_inner())?)
        };
        Ok::<UserRead, CustomError>(user)
    })
    .await??;

    return Ok(Success::new(user));
}

#[post("/users")]
async fn create(
    pool: web::Data<DbPool>,
    _: Auth,
    user: web::Json<NewUser>,
) -> actix_web::Result<impl Responder> {
    if let Some(email) = &user.email {
        if email.is_empty() {
            return Err(CustomError::new(400, "Invalid email"))?;
        }

        if let Some(password) = &user.password {
            if password.is_empty() {
                return Err(CustomError::new(400, "Invalid password"))?;
            }
        } else {
            return Err(CustomError::new(400, "Invalid password"))?;
        }
    } else {
        if let Some(_) = &user.password {
            return Err(CustomError::new(400, "Invalid email or password"))?;
        }
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
    if auth.user_id != user_id.clone() {
        return Err(CustomError::new(401, "Missing permissions for this user"))?;
    }

    let user = web::block(move || {
        let mut conn = pool.get()?;
        User::update(&mut conn, user_id.into_inner(), user.into_inner())
    })
    .await??;

    Ok(Success::new(user))
}
