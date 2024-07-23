// @generated automatically by Diesel CLI.

diesel::table! {
    review_company (review_id, user_id) {
        review_id -> Uuid,
        user_id -> Uuid,
    }
}

diesel::table! {
    reviews (review_id) {
        review_id -> Uuid,
        user_id -> Uuid,
        media_id -> Int4,
        imdb_id -> Nullable<Text>,
        media_type -> Text,
        media_title -> Text,
        media_poster_uri -> Nullable<Text>,
        media_release_year -> Int2,
        date -> Nullable<Date>,
        rating -> Int2,
        review_title -> Nullable<Text>,
        review_description -> Nullable<Text>,
        venue -> Nullable<Text>,
    }
}

diesel::table! {
    users (user_id) {
        user_id -> Uuid,
        first_name -> Text,
        last_name -> Text,
        email -> Text,
        avatar_uri -> Nullable<Text>,
        date_created -> Date,
        permission_level -> Int2,
        public -> Bool,
    }
}

diesel::joinable!(review_company -> reviews (review_id));
diesel::joinable!(review_company -> users (user_id));
diesel::joinable!(reviews -> users (user_id));

diesel::allow_tables_to_appear_in_same_query!(
    review_company,
    reviews,
    users,
);
