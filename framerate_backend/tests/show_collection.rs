pub mod common;

mod find {
    use crate::common::{data, process, setup};
    use actix_web::{http::header::AUTHORIZATION, test};
    use framerate::show_collection::{find, ShowCollection};

    #[actix_web::test]
    async fn should_require_authentication() {
        let (app, pool) = setup::create_app(find).await;

        let collection = {
            let mut conn = pool.get().unwrap();
            let user = data::create_user(&mut conn);
            data::create_show_collection(&mut conn, &user)
        };

        let request = test::TestRequest::get()
            .uri(&format!("/shows/collections/{}", collection.collection_id))
            .to_request();

        let response = test::call_service(&app, request).await;
        assert_eq!(401, response.status());
    }

    #[actix_web::test]
    async fn should_not_return_other_users_show_collection() {
        let (app, pool) = setup::create_app(find).await;

        let (token, show_collection) = {
            let mut conn = pool.get().unwrap();
            let user = data::create_user(&mut conn);
            let show_collection = data::create_show_collection(&mut conn, &user);
            let (token, _) = data::create_authed_user(&mut conn);
            (token, show_collection)
        };

        let request = test::TestRequest::get()
            .uri(&format!(
                "/shows/collections/{}",
                show_collection.collection_id
            ))
            .insert_header((AUTHORIZATION, format!("Bearer {token}")))
            .to_request();

        let response = test::call_service(&app, request).await;
        assert_eq!(404, response.status());
    }

    #[actix_web::test]
    async fn should_return_show_collection() {
        let (app, pool) = setup::create_app(find).await;

        let (token, collection, show_entry) = {
            let mut conn = pool.get().unwrap();
            let (token, user) = data::create_authed_user(&mut conn);
            let collection = data::create_show_collection(&mut conn, &user);
            let show_entry = data::create_show_entry(&mut conn, &user, &collection);
            (token, collection, show_entry)
        };

        let request = test::TestRequest::get()
            .uri(&format!("/shows/collections/{}", collection.collection_id,))
            .insert_header((AUTHORIZATION, format!("Bearer {token}")))
            .to_request();

        let response = test::call_service(&app, request).await;
        assert!(response.status().is_success());

        let result = process::parse_body::<ShowCollection>(response).await;

        let returned_entry = result.data.entries.unwrap();

        assert_eq!(collection.name, result.data.name);
        assert_eq!(&show_entry, returned_entry.first().unwrap());
    }
}

mod find_all {
    use crate::common::{data, process, setup};
    use actix_web::{http::header::AUTHORIZATION, test};
    use framerate::show_collection::{find_all, ShowCollection};

    #[actix_web::test]
    async fn should_require_authentication() {
        let (app, pool) = setup::create_app(find_all).await;

        {
            let mut conn = pool.get().unwrap();
            let user = data::create_user(&mut conn);
            data::create_show_collection(&mut conn, &user)
        };

        let request = test::TestRequest::get()
            .uri("/shows/collections")
            .to_request();

        let response = test::call_service(&app, request).await;
        assert_eq!(401, response.status());
    }

    #[actix_web::test]
    async fn should_not_return_other_users_show_collections() {
        let (app, pool) = setup::create_app(find_all).await;

        let token = {
            let mut conn = pool.get().unwrap();
            let user = data::create_user(&mut conn);
            let collection = data::create_show_collection(&mut conn, &user);
            data::create_show_entry(&mut conn, &user, &collection);
            let (token, _) = data::create_authed_user(&mut conn);
            token
        };

        let request = test::TestRequest::get()
            .uri("/shows/collections")
            .insert_header((AUTHORIZATION, format!("Bearer {token}")))
            .to_request();

        let response = test::call_service(&app, request).await;
        assert!(response.status().is_success());

        let result = process::parse_body::<Vec<ShowCollection>>(response).await;
        assert_eq!(0, result.data.len());
    }

    #[actix_web::test]
    async fn should_return_user_show_collections() {
        let (app, pool) = setup::create_app(find_all).await;

        let (token, collection) = {
            let mut conn = pool.get().unwrap();
            let (token, user) = data::create_authed_user(&mut conn);
            let collection = data::create_show_collection(&mut conn, &user);
            (token, collection)
        };

        let request = test::TestRequest::get()
            .uri("/shows/collections")
            .insert_header((AUTHORIZATION, format!("Bearer {token}")))
            .to_request();

        let response = test::call_service(&app, request).await;
        assert!(response.status().is_success());

        let result = process::parse_body::<Vec<ShowCollection>>(response).await;
        assert_eq!(1, result.data.len());

        let returned_collection = result.data.first().unwrap();
        assert_eq!(collection.collection_id, returned_collection.collection_id);
        assert_eq!(collection.name, returned_collection.name);
        assert_eq!(None, returned_collection.entries);
    }
}

mod create {
    use crate::common::{data, process, setup};
    use actix_web::{http::header::AUTHORIZATION, test};
    use framerate::show_collection::{create, ShowCollection};

    #[actix_web::test]
    async fn should_require_authentication() {
        let (app, pool) = setup::create_app(create).await;

        {
            let mut conn = pool.get().unwrap();
            data::create_user(&mut conn);
        };

        let show_collection = data::generate_save_show_collection();

        let request = test::TestRequest::post()
            .uri("/shows/collections")
            .set_json(&show_collection)
            .to_request();

        let response = test::call_service(&app, request).await;
        assert_eq!(401, response.status());
    }

    #[actix_web::test]
    async fn should_create_show_collection() {
        let (app, pool) = setup::create_app(create).await;

        let (token, user) = {
            let mut conn = pool.get().unwrap();
            data::create_authed_user(&mut conn)
        };

        let show_collection = data::generate_save_show_collection();

        let request = test::TestRequest::post()
            .uri("/shows/collections")
            .set_json(&show_collection)
            .insert_header((AUTHORIZATION, format!("Bearer {token}")))
            .to_request();

        let response = test::call_service(&app, request).await;
        assert!(response.status().is_success());

        let result = process::parse_body::<ShowCollection>(response).await;
        let returned_show_collection = result.data;

        assert!(!returned_show_collection.collection_id.is_nil());
        assert_eq!(show_collection.name, returned_show_collection.name);
        assert_eq!(user.user_id, returned_show_collection.user_id);
    }
}

mod update {
    use crate::common::{data, process, setup};
    use actix_web::{http::header::AUTHORIZATION, test};
    use framerate::show_collection::{update, ShowCollection};

    #[actix_web::test]
    async fn should_require_authentication() {
        let (app, pool) = setup::create_app(update).await;

        let collection = {
            let mut conn = pool.get().unwrap();
            let user = data::create_user(&mut conn);
            data::create_show_collection(&mut conn, &user)
        };

        let updated_collection = data::generate_update_show_collection();

        let request = test::TestRequest::put()
            .uri(&format!("/shows/collections/{}", collection.collection_id,))
            .set_json(&updated_collection)
            .to_request();

        let response = test::call_service(&app, request).await;
        assert_eq!(401, response.status());
    }

    #[actix_web::test]
    async fn should_not_update_other_users_show_collection() {
        let (app, pool) = setup::create_app(update).await;

        let (token, collection) = {
            let mut conn = pool.get().unwrap();
            let user = data::create_user(&mut conn);
            let collection = data::create_show_collection(&mut conn, &user);
            let (token, _) = data::create_authed_user(&mut conn);
            (token, collection)
        };

        let updated_collection = data::generate_update_show_collection();

        let request = test::TestRequest::put()
            .uri(&format!("/shows/collections/{}", collection.collection_id,))
            .insert_header((AUTHORIZATION, format!("Bearer {token}")))
            .set_json(&updated_collection)
            .to_request();

        let response = test::call_service(&app, request).await;
        assert_eq!(404, response.status());
    }

    #[actix_web::test]
    async fn should_update_show_collection() {
        let (app, pool) = setup::create_app(update).await;

        let (token, collection) = {
            let mut conn = pool.get().unwrap();
            let (token, user) = data::create_authed_user(&mut conn);
            let collection = data::create_show_collection(&mut conn, &user);
            (token, collection)
        };

        let updated_collection = data::generate_update_show_collection();

        let request = test::TestRequest::put()
            .uri(&format!("/shows/collections/{}", collection.collection_id,))
            .insert_header((AUTHORIZATION, format!("Bearer {token}")))
            .set_json(&updated_collection)
            .to_request();

        let response = test::call_service(&app, request).await;
        assert!(response.status().is_success());

        let result = process::parse_body::<ShowCollection>(response).await;

        assert_eq!(collection.collection_id, result.data.collection_id);
        assert_eq!(collection.user_id, result.data.user_id);
        assert_eq!(updated_collection.name, result.data.name);
    }
}

mod delete {
    use crate::common::{data, process, setup};
    use actix_web::{http::header::AUTHORIZATION, test};
    use framerate::{show_collection::delete, utils::response_body::DeleteResponse};

    #[actix_web::test]
    async fn should_require_authentication() {
        let (app, pool) = setup::create_app(delete).await;

        let collection = {
            let mut conn = pool.get().unwrap();
            let user = data::create_user(&mut conn);
            data::create_show_collection(&mut conn, &user)
        };

        let request = test::TestRequest::delete()
            .uri(&format!("/shows/collections/{}", collection.collection_id,))
            .to_request();

        let response = test::call_service(&app, request).await;
        assert_eq!(401, response.status());
    }

    #[actix_web::test]
    async fn should_not_delete_other_users_show_collection() {
        let (app, pool) = setup::create_app(delete).await;

        let (token, collection) = {
            let mut conn = pool.get().unwrap();
            let user = data::create_user(&mut conn);
            let collection = data::create_show_collection(&mut conn, &user);
            let (token, _) = data::create_authed_user(&mut conn);
            (token, collection)
        };

        let request = test::TestRequest::delete()
            .uri(&format!("/shows/collections/{}", collection.collection_id,))
            .insert_header((AUTHORIZATION, format!("Bearer {token}")))
            .to_request();

        let response = test::call_service(&app, request).await;
        assert_eq!(404, response.status());
    }

    #[actix_web::test]
    async fn should_delete_show_collection() {
        let (app, pool) = setup::create_app(delete).await;

        let (token, collection) = {
            let mut conn = pool.get().unwrap();
            let (token, user) = data::create_authed_user(&mut conn);
            let collection = data::create_show_collection(&mut conn, &user);
            (token, collection)
        };

        let request = test::TestRequest::delete()
            .uri(&format!("/shows/collections/{}", collection.collection_id,))
            .insert_header((AUTHORIZATION, format!("Bearer {token}")))
            .to_request();

        let response = test::call_service(&app, request).await;
        assert!(response.status().is_success());

        let result = process::parse_body::<DeleteResponse>(response).await;

        assert_eq!(1, result.data.count);
    }
}

mod create_entry {
    use crate::common::{data, process, setup};
    use actix_web::{http::header::AUTHORIZATION, test};
    use framerate::{show_collection::create_entry, show_entry::ShowEntry};

    #[actix_web::test]
    async fn should_require_authentication() {
        let (app, pool) = setup::create_app(create_entry).await;

        let collection = {
            let mut conn = pool.get().unwrap();
            let user = data::create_user(&mut conn);
            let collection = data::create_show_collection(&mut conn, &user);
            collection
        };

        let show_entry = data::generate_save_show_collection_entry();

        let request = test::TestRequest::post()
            .uri(&format!("/shows/collections/{}", collection.collection_id))
            .set_json(&show_entry)
            .to_request();

        let response = test::call_service(&app, request).await;
        assert_eq!(401, response.status());
    }

    #[actix_web::test]
    async fn should_not_create_show_entry_on_other_users_collection() {
        let (app, pool) = setup::create_app(create_entry).await;

        let (token, collection) = {
            let mut conn = pool.get().unwrap();
            let user = data::create_user(&mut conn);
            let collection = data::create_show_collection(&mut conn, &user);
            let (token, _) = data::create_authed_user(&mut conn);
            (token, collection)
        };

        let show_entry = data::generate_save_show_collection_entry();

        let request = test::TestRequest::post()
            .uri(&format!("/shows/collections/{}", collection.collection_id))
            .set_json(&show_entry)
            .insert_header((AUTHORIZATION, format!("Bearer {token}")))
            .to_request();

        let response = test::call_service(&app, request).await;
        assert_eq!(404, response.status());
    }

    #[actix_web::test]
    async fn should_create_show_entry() {
        let (app, pool) = setup::create_app(create_entry).await;

        let (token, user, collection) = {
            let mut conn = pool.get().unwrap();
            let (token, user) = data::create_authed_user(&mut conn);
            let collection = data::create_show_collection(&mut conn, &user);
            (token, user, collection)
        };

        let show_entry = data::generate_save_show_collection_entry();

        let request = test::TestRequest::post()
            .uri(&format!("/shows/collections/{}", collection.collection_id))
            .set_json(&show_entry)
            .insert_header((AUTHORIZATION, format!("Bearer {token}")))
            .to_request();

        let response = test::call_service(&app, request).await;
        assert!(response.status().is_success());

        let result = process::parse_body::<ShowEntry>(response).await;
        let returned_show_entry = result.data;

        assert!(!returned_show_entry.collection_id.is_nil());
        assert_eq!(show_entry.show_id, returned_show_entry.show_id);
        assert_eq!(user.user_id, returned_show_entry.user_id);
    }
}

mod delete_entry {
    use crate::common::{data, process, setup};
    use actix_web::{http::header::AUTHORIZATION, test};
    use framerate::{show_collection::delete_entry, utils::response_body::DeleteResponse};

    #[actix_web::test]
    async fn should_require_authentication() {
        let (app, pool) = setup::create_app(delete_entry).await;

        let show_entry = {
            let mut conn = pool.get().unwrap();
            let user = data::create_user(&mut conn);
            let collection = data::create_show_collection(&mut conn, &user);
            data::create_show_entry(&mut conn, &user, &collection)
        };

        let request = test::TestRequest::delete()
            .uri(&format!(
                "/shows/collections/{}/{}",
                show_entry.collection_id, show_entry.show_id
            ))
            .to_request();

        let response = test::call_service(&app, request).await;
        assert_eq!(401, response.status());
    }

    #[actix_web::test]
    async fn should_not_delete_other_users_show_entry() {
        let (app, pool) = setup::create_app(delete_entry).await;

        let (token, show_entry) = {
            let mut conn = pool.get().unwrap();
            let user = data::create_user(&mut conn);
            let collection = data::create_show_collection(&mut conn, &user);
            let show_entry = data::create_show_entry(&mut conn, &user, &collection);
            let (token, _) = data::create_authed_user(&mut conn);
            (token, show_entry)
        };

        let request = test::TestRequest::delete()
            .uri(&format!(
                "/shows/collections/{}/{}",
                show_entry.collection_id, show_entry.show_id
            ))
            .insert_header((AUTHORIZATION, format!("Bearer {token}")))
            .to_request();

        let response = test::call_service(&app, request).await;
        assert_eq!(404, response.status());
    }

    #[actix_web::test]
    async fn should_delete_show_entry() {
        let (app, pool) = setup::create_app(delete_entry).await;

        let (token, show_entry) = {
            let mut conn = pool.get().unwrap();
            let (token, user) = data::create_authed_user(&mut conn);
            let collection = data::create_show_collection(&mut conn, &user);
            let show_entry = data::create_show_entry(&mut conn, &user, &collection);
            (token, show_entry)
        };

        let request = test::TestRequest::delete()
            .uri(&format!(
                "/shows/collections/{}/{}",
                show_entry.collection_id, show_entry.show_id
            ))
            .insert_header((AUTHORIZATION, format!("Bearer {token}")))
            .to_request();

        let response = test::call_service(&app, request).await;
        assert!(response.status().is_success());

        let result = process::parse_body::<DeleteResponse>(response).await;

        assert_eq!(1, result.data.count);
    }
}

mod find_by_show {
    use crate::common::{data, process, setup};
    use actix_web::{http::header::AUTHORIZATION, test};
    use framerate::show_collection::find_by_show;
    use uuid::Uuid;

    #[actix_web::test]
    async fn should_require_authentication() {
        let (app, pool) = setup::create_app(find_by_show).await;

        let show_entry = {
            let mut conn = pool.get().unwrap();
            let user = data::create_user(&mut conn);
            let collection = data::create_show_collection(&mut conn, &user);
            data::create_show_entry(&mut conn, &user, &collection)
        };

        let request = test::TestRequest::get()
            .uri(&format!("/shows/collections/show/{}", show_entry.show_id))
            .to_request();

        let response = test::call_service(&app, request).await;
        assert_eq!(401, response.status());
    }

    #[actix_web::test]
    async fn should_not_return_other_users_show_collections() {
        let (app, pool) = setup::create_app(find_by_show).await;

        let (token, show_entry) = {
            let mut conn = pool.get().unwrap();
            let user = data::create_user(&mut conn);
            let collection = data::create_show_collection(&mut conn, &user);
            let show_entry = data::create_show_entry(&mut conn, &user, &collection);
            let (token, _) = data::create_authed_user(&mut conn);
            (token, show_entry)
        };

        let request = test::TestRequest::get()
            .uri(&format!("/shows/collections/show/{}", show_entry.show_id))
            .insert_header((AUTHORIZATION, format!("Bearer {token}")))
            .to_request();

        let response = test::call_service(&app, request).await;
        assert!(response.status().is_success());

        let result = process::parse_body::<Vec<Uuid>>(response).await;
        assert_eq!(0, result.data.len());
    }

    #[actix_web::test]
    async fn should_return_user_show_collections() {
        let (app, pool) = setup::create_app(find_by_show).await;

        let (token, collection, show_entry) = {
            let mut conn = pool.get().unwrap();
            let (token, user) = data::create_authed_user(&mut conn);
            let collection = data::create_show_collection(&mut conn, &user);
            let show_entry = data::create_show_entry(&mut conn, &user, &collection);
            (token, collection, show_entry)
        };

        let request = test::TestRequest::get()
            .uri(&format!("/shows/collections/show/{}", show_entry.show_id))
            .insert_header((AUTHORIZATION, format!("Bearer {token}")))
            .to_request();

        let response = test::call_service(&app, request).await;
        assert!(response.status().is_success());

        let result = process::parse_body::<Vec<Uuid>>(response).await;
        assert_eq!(1, result.data.len());

        let returned_collection_id = result.data.first().unwrap();
        assert_eq!(&collection.collection_id, returned_collection_id);
    }
}
