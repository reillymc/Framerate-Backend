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
        email -> Text,
        password -> Text,
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

diesel::joinable!(review_company -> reviews (review_id));
diesel::joinable!(review_company -> users (user_id));
diesel::joinable!(reviews -> users (user_id));
diesel::joinable!(watchlist_entries -> users (user_id));
diesel::joinable!(watchlist_entries -> watchlists (watchlist_id));
diesel::joinable!(watchlists -> users (user_id));

diesel::allow_tables_to_appear_in_same_query!(
    review_company,
    reviews,
    users,
    watchlist_entries,
    watchlists,
);
