-- Your SQL goes here
CREATE TABLE "movie_reviews"(
    "review_id" uuid NOT NULL PRIMARY KEY,
    "user_id" uuid NOT NULL,
    "movie_id" int4 NOT NULL,
    "title" text NOT NULL,
    "imdb_id" text,
    "poster_path" text,
    "release_date" date,
    FOREIGN KEY ("review_id") REFERENCES "reviews"("review_id"),
    FOREIGN KEY ("user_id") REFERENCES "users"("user_id")
);

INSERT INTO "movie_reviews"(review_id, user_id, movie_id, title, imdb_id, poster_path, release_date)
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
    media_type = 'movie';

