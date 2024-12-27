use crate::{
    company::model::{Company, SaveCompany},
    db::DbPool,
    utils::{
        jwt::Auth,
        response_body::{DeleteResponse, Success},
        AppError,
    },
};
use actix_web::{delete, get, post, put, web, Responder};
use uuid::Uuid;

#[get("/company")]
async fn find_all(pool: web::Data<DbPool>, auth: Auth) -> actix_web::Result<impl Responder> {
    let company = web::block(move || {
        let mut conn = pool.get()?;
        Company::find_all(&mut conn, &auth.user_id)
    })
    .await??;

    Ok(Success::new(company))
}

#[post("/company")]
async fn create(
    pool: web::Data<DbPool>,
    auth: Auth,
    company: web::Json<SaveCompany>,
) -> actix_web::Result<impl Responder> {
    let company = web::block(move || {
        let mut conn = pool.get()?;
        Company::create(&mut conn, company.into_inner(), auth.user_id)
    })
    .await??;

    Ok(Success::new(company))
}

#[put("/company/{user_id}")]
async fn update(
    pool: web::Data<DbPool>,
    auth: Auth,
    user_id: web::Path<Uuid>,
    company: web::Json<SaveCompany>,
) -> actix_web::Result<impl Responder> {
    let company = web::block(move || {
        let mut conn = pool.get()?;
        Company::update(
            &mut conn,
            user_id.into_inner(),
            company.into_inner(),
            &auth.user_id,
        )
    })
    .await?;

    let Ok(company) = company else {
        return Err(AppError::external(404, "Company not found"))?;
    };

    Ok(Success::new(company))
}

#[delete("/company/{user_id}")]
async fn delete(
    pool: web::Data<DbPool>,
    auth: Auth,
    user_id: web::Path<Uuid>,
) -> actix_web::Result<impl Responder> {
    let count = web::block(move || {
        let mut conn = pool.get()?;
        Company::delete(&mut conn, user_id.into_inner(), &auth.user_id)
    })
    .await??;

    if count == 0 {
        return Err(AppError::external(404, "Company not found"))?;
    }

    Ok(Success::new(DeleteResponse { count }))
}
