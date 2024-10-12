// @generated automatically by Diesel CLI.

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
        rating -> Int2,
        title -> Nullable<Text>,
        description -> Nullable<Text>,
        venue -> Nullable<Text>,
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
    }
}

diesel::table! {
    watchlist_entries (watchlist_id, media_id) {
        watchlist_id -> Uuid,
        media_id -> Int4,
        imdb_id -> Nullable<Text>,
        user_id -> Uuid,
        media_type -> Text,
        media_title -> Text,
        media_poster_uri -> Nullable<Text>,
        media_release_date -> Nullable<Date>,
    }
}

diesel::table! {
    watchlists (watchlist_id) {
        watchlist_id -> Uuid,
        user_id -> Uuid,
        name -> Text,
        media_type -> Text,
    }
}

diesel::joinable!(movie_reviews -> reviews (review_id));
diesel::joinable!(movie_reviews -> users (user_id));
diesel::joinable!(review_company -> reviews (review_id));
diesel::joinable!(review_company -> users (user_id));
diesel::joinable!(reviews -> users (user_id));
diesel::joinable!(show_reviews -> reviews (review_id));
diesel::joinable!(show_reviews -> users (user_id));
diesel::joinable!(watchlist_entries -> users (user_id));
diesel::joinable!(watchlist_entries -> watchlists (watchlist_id));
diesel::joinable!(watchlists -> users (user_id));

diesel::allow_tables_to_appear_in_same_query!(
    movie_reviews,
    review_company,
    reviews,
    show_reviews,
    users,
    watchlist_entries,
    watchlists,
);
