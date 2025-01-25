use super::{ClientConfig, MetaEntry, MetaEntryKey};
use crate::db::DbPool;
use crate::utils::AppError;
use crate::utils::{jwt::Auth, response_body::Success};
use actix_web::{get, put, web, Responder};

#[utoipa::path(tag = "Meta", responses((status = OK, body = ClientConfig)))]
#[get("/meta/client_config")]
async fn get_client_config(pool: web::Data<DbPool>, _: Auth) -> actix_web::Result<impl Responder> {
    let entry = web::block(move || {
        let mut conn = pool.get()?;
        MetaEntry::find(&mut conn, MetaEntryKey::ClientConfig)
    })
    .await??;

    Ok(Success::new(entry))
}

#[utoipa::path(tag = "Meta", responses((status = UNAUTHORIZED),(status = OK, body = ClientConfig)))]
#[put("/meta/client_config")]
async fn update_client_config(
    pool: web::Data<DbPool>,
    auth: Auth,
    client_config: web::Json<ClientConfig>,
) -> actix_web::Result<impl Responder> {
    if !auth.is_at_least_admin() {
        return Err(AppError::external(
            401,
            "Unauthorized to update client config",
        ))?;
    }

    let entry = web::block(move || {
        let mut conn = pool.get()?;
        MetaEntry::update(
            &mut conn,
            MetaEntry::ClientConfig(client_config.into_inner()),
        )
    })
    .await??;

    Ok(Success::new(entry))
}
