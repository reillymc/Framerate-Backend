-- -- Your SQL goes here
CREATE TABLE "movie_entries"(
    "watchlist_id" uuid NOT NULL,
    "movie_id" int4 NOT NULL,
    "user_id" uuid NOT NULL,
    "title" text NOT NULL,
    "imdb_id" text,
    "poster_path" text,
    "release_date" date,
    PRIMARY KEY ("watchlist_id", "movie_id"),
    FOREIGN KEY ("watchlist_id") REFERENCES "watchlists"("watchlist_id"),
    FOREIGN KEY ("user_id") REFERENCES "users"("user_id")
);

CREATE TABLE "show_entries"(
    "watchlist_id" uuid NOT NULL,
    "show_id" int4 NOT NULL,
    "user_id" uuid NOT NULL,
    "name" text NOT NULL,
    "updated_at" date NOT NULL DEFAULT CURRENT_DATE,
    "imdb_id" text,
    "status" text,
    "poster_path" text,
    "first_air_date" date,
    "last_air_date" date,
    "next_air_date" date,
    PRIMARY KEY ("watchlist_id", "show_id"),
    FOREIGN KEY ("watchlist_id") REFERENCES "watchlists"("watchlist_id"),
    FOREIGN KEY ("user_id") REFERENCES "users"("user_id")
);

INSERT INTO "movie_entries"(watchlist_id, movie_id, user_id, title, imdb_id, poster_path, release_date)
SELECT
    watchlist_id,
    media_id,
    user_id,
    media_title,
    imdb_id,
    media_poster_uri,
    media_release_date
FROM
    watchlist_entries
WHERE
    media_type = 'movie';

INSERT INTO "show_entries"(watchlist_id, show_id, user_id, name, imdb_id, poster_path, first_air_date)
SELECT
    watchlist_id,
    media_id,
    user_id,
    media_title,
    imdb_id,
    media_poster_uri,
    media_release_date
FROM
    watchlist_entries
WHERE
    media_type = 'show';

DROP TABLE IF EXISTS "watchlist_entries";

