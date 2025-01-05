// @generated automatically by Diesel CLI.

diesel::table! {
    collections (collection_id) {
        collection_id -> Uuid,
        user_id -> Uuid,
        name -> Text,
        media_type -> Text,
        default_for -> Nullable<Text>,
    }
}

diesel::table! {
    movie_entries (collection_id, movie_id) {
        collection_id -> Uuid,
        movie_id -> Int4,
        user_id -> Uuid,
        title -> Text,
        imdb_id -> Nullable<Text>,
        poster_path -> Nullable<Text>,
        release_date -> Nullable<Date>,
        status -> Nullable<Text>,
        updated_at -> Date,
    }
}

diesel::table! {
    movie_reviews (review_id) {
        review_id -> Uuid,
        user_id -> Uuid,
        movie_id -> Int4,
        title -> Text,
        imdb_id -> Nullable<Text>,
        poster_path -> Nullable<Text>,
        release_date -> Nullable<Date>,
    }
}

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
        date -> Nullable<Date>,
        rating -> Nullable<Int2>,
        title -> Nullable<Text>,
        description -> Nullable<Text>,
        venue -> Nullable<Text>,
    }
}

diesel::table! {
    season_reviews (review_id) {
        review_id -> Uuid,
        user_id -> Uuid,
        show_id -> Int4,
        season_number -> Int4,
        name -> Nullable<Text>,
        poster_path -> Nullable<Text>,
        air_date -> Nullable<Date>,
    }
}

diesel::table! {
    show_entries (collection_id, show_id) {
        collection_id -> Uuid,
        show_id -> Int4,
        user_id -> Uuid,
        name -> Text,
        updated_at -> Date,
        imdb_id -> Nullable<Text>,
        status -> Nullable<Text>,
        poster_path -> Nullable<Text>,
        first_air_date -> Nullable<Date>,
        last_air_date -> Nullable<Date>,
        next_air_date -> Nullable<Date>,
    }
}

diesel::table! {
    show_reviews (review_id) {
        review_id -> Uuid,
        user_id -> Uuid,
        show_id -> Int4,
        name -> Text,
        imdb_id -> Nullable<Text>,
        poster_path -> Nullable<Text>,
        first_air_date -> Nullable<Date>,
    }
}

diesel::table! {
    users (user_id) {
        user_id -> Uuid,
        email -> Nullable<Text>,
        password -> Nullable<Text>,
        first_name -> Text,
        last_name -> Text,
        avatar_uri -> Nullable<Text>,
        date_created -> Timestamp,
        permission_level -> Int2,
        public -> Bool,
        configuration -> Jsonb,
        created_by -> Nullable<Uuid>,
    }
}

diesel::joinable!(collections -> users (user_id));
diesel::joinable!(movie_entries -> collections (collection_id));
diesel::joinable!(movie_entries -> users (user_id));
diesel::joinable!(movie_reviews -> reviews (review_id));
diesel::joinable!(movie_reviews -> users (user_id));
diesel::joinable!(review_company -> reviews (review_id));
diesel::joinable!(review_company -> users (user_id));
diesel::joinable!(reviews -> users (user_id));
diesel::joinable!(season_reviews -> reviews (review_id));
diesel::joinable!(season_reviews -> users (user_id));
diesel::joinable!(show_entries -> collections (collection_id));
diesel::joinable!(show_entries -> users (user_id));
diesel::joinable!(show_reviews -> reviews (review_id));
diesel::joinable!(show_reviews -> users (user_id));

diesel::allow_tables_to_appear_in_same_query!(
    collections,
    movie_entries,
    movie_reviews,
    review_company,
    reviews,
    season_reviews,
    show_entries,
    show_reviews,
    users,
);
