use crate::{
    season::Season,
    tmdb::TmdbClient,
    utils::{
        jwt::Auth,
        response_body::{Error, Success},
    },
};
use actix_web::{get, web, HttpResponse, Responder};

#[get("/shows/{show_id}/seasons/{season_number}/details")]
async fn find(
    _: Auth,
    client: web::Data<TmdbClient>,
    path: web::Path<(i32, i32)>,
) -> impl Responder {
    let (show_id, season_number) = path.into_inner();

    let show = Season::find(&client, &show_id, &season_number).await;

    match show {
        Ok(_) => HttpResponse::Ok().json(Success::new(show.unwrap())),
        Err(err) => HttpResponse::InternalServerError().json(Error {
            message: err.to_string(),
        }),
    }
}
