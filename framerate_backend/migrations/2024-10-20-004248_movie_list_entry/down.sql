-- This file should undo anything in `up.sql`
CREATE TABLE "watchlist_entries"(
    "watchlist_id" uuid NOT NULL,
    "media_id" int4 NOT NULL,
    "imdb_id" text,
    "user_id" uuid NOT NULL,
    "media_type" text NOT NULL,
    "media_title" text NOT NULL,
    "media_poster_uri" text,
    "media_release_date" date,
    PRIMARY KEY ("watchlist_id", "media_id"),
    FOREIGN KEY ("watchlist_id") REFERENCES "watchlists"("watchlist_id"),
    FOREIGN KEY ("user_id") REFERENCES "users"("user_id")
);

INSERT INTO "watchlist_entries"(watchlist_id, media_id, imdb_id, user_id, media_type, media_title, media_poster_uri, media_release_date)
SELECT
    watchlist_id,
    movie_id,
    imdb_id,
    user_id,
    'movie',
    title,
    poster_path,
    release_date
FROM
    movie_entries;

INSERT INTO "watchlist_entries"(watchlist_id, media_id, imdb_id, user_id, media_type, media_title, media_poster_uri, media_release_date)
SELECT
    watchlist_id,
    show_id,
    imdb_id,
    user_id,
    'show',
    name,
    poster_path,
    first_air_date
FROM
    show_entries;

DROP TABLE IF EXISTS "movie_entries";

DROP TABLE IF EXISTS "show_entries";

