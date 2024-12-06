pub mod setup {
    use actix_http::Request;
    use actix_web::{
        dev::{HttpServiceFactory, Service, ServiceResponse},
        test,
        web::Data,
        App, Error,
    };
    use diesel::{r2d2::ConnectionManager, PgConnection};
    use framerate::db;
    use r2d2::PooledConnection;

    pub async fn create_app<F>(
        service: F,
    ) -> (
        impl Service<Request, Response = ServiceResponse, Error = Error>,
        PooledConnection<ConnectionManager<PgConnection>>,
    )
    where
        F: HttpServiceFactory + 'static,
    {
        let pool = db::get_connection_pool();
        let conn = pool.get().unwrap();
        let app = test::init_service(
            App::new()
                .app_data(Data::new(pool.clone()))
                .service(service),
        )
        .await;

        (app, conn)
    }
}

pub mod data {
    use diesel::{r2d2::ConnectionManager, PgConnection};
    use r2d2::PooledConnection;
    use uuid::Uuid;

    use framerate::{
        user::{self, NewUser, User},
        utils::jwt::create_token,
    };

    pub fn get_authed_user(
        conn: &mut PooledConnection<ConnectionManager<PgConnection>>,
    ) -> (User, String) {
        let user = user::User::create(
            conn,
            NewUser {
                first_name: Uuid::new_v4().to_string(),
                last_name: Uuid::new_v4().to_string(),
                avatar_uri: Some(Uuid::new_v4().to_string()),
                email: Some(Uuid::new_v4().to_string()),
                password: Some(Uuid::new_v4().to_string()),
                configuration: None,
                user_id: None,
            },
        )
        .unwrap();

        let token = create_token(user.user_id, &user.email.clone().unwrap()).unwrap();

        (user, token)
    }
}

pub mod process {
    use actix_web::dev::ServiceResponse;
    use actix_web::test;
    use framerate::utils::response_body::Success;
    use serde::Deserialize;

    pub async fn parse_body<T: for<'a> Deserialize<'a>>(response: ServiceResponse) -> Success<T> {
        let body = test::read_body(response).await;
        let data: Success<T> = serde_json::from_slice(&body).unwrap();
        data
    }
}
