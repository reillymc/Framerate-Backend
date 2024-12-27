-- Your SQL goes here
CREATE TABLE "show_reviews"(
    "review_id" uuid NOT NULL PRIMARY KEY,
    "user_id" uuid NOT NULL,
    "show_id" int4 NOT NULL,
    "name" text NOT NULL,
    "imdb_id" text,
    "poster_path" text,
    "first_air_date" date,
    FOREIGN KEY ("review_id") REFERENCES "reviews"("review_id"),
    FOREIGN KEY ("user_id") REFERENCES "users"("user_id")
);

INSERT INTO "show_reviews"(review_id, user_id, show_id, name, imdb_id, poster_path, first_air_date)
SELECT
    review_id,
    user_id,
    media_id,
    media_title,
    imdb_id,
    media_poster_uri,
    media_release_date
FROM
    reviews
WHERE
    media_type = 'show';

