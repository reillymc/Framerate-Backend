// @generated automatically by Diesel CLI.

diesel::table! {
    ratings (rating_id) {
        rating_id -> Uuid,
        user_id -> Uuid,
        movie_id -> Int4,
        movie_title -> Text,
        movie_poster_uri -> Text,
        movie_release_year -> Int2,
        date -> Date,
        value -> Int2,
        review_title -> Nullable<Text>,
        review_description -> Nullable<Text>,
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

diesel::joinable!(ratings -> users (user_id));

diesel::allow_tables_to_appear_in_same_query!(
    ratings,
    users,
);
