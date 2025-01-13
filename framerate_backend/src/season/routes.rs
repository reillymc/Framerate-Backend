use crate::{
    season::Season,
    tmdb::TmdbClient,
    utils::{jwt::Auth, response_body::Success},
};
use actix_web::{get, web, Responder};

#[utoipa::path(tag = "Season", responses((status = OK, body = Season),(status = NOT_FOUND)))]
#[get("/shows/{show_id}/seasons/{season_number}/details")]
async fn details(
    _: Auth,
    client: web::Data<TmdbClient>,
    path: web::Path<(i32, i32)>,
) -> actix_web::Result<impl Responder> {
    let (show_id, season_number) = path.into_inner();

    let show = Season::find(&client, &show_id, &season_number).await?;
    Ok(Success::new(show))
}
