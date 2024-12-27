-- Your SQL goes here
CREATE TABLE "watchlists"(
    "watchlist_id" UUID NOT NULL PRIMARY KEY,
    "user_id" UUID NOT NULL,
    "name" TEXT NOT NULL,
    "media_type" TEXT NOT NULL,
    FOREIGN KEY ("user_id") REFERENCES "users"("user_id")
);

CREATE TABLE "watchlist_entries"(
    "watchlist_id" UUID NOT NULL,
    "media_id" INT4 NOT NULL,
    "imdb_id" TEXT,
    "user_id" UUID NOT NULL,
    "media_type" TEXT NOT NULL,
    "media_title" TEXT NOT NULL,
    "media_poster_uri" TEXT,
    "media_release_date" DATE,
    PRIMARY KEY ("watchlist_id", "media_id"),
    FOREIGN KEY ("watchlist_id") REFERENCES "watchlists"("watchlist_id"),
    FOREIGN KEY ("user_id") REFERENCES "users"("user_id")
);
