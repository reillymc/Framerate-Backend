-- This file should undo anything in `up.sql`
ALTER TABLE "reviews"
    ADD COLUMN "media_id" INT4,
    ADD COLUMN "imdb_id" TEXT,
    ADD COLUMN "media_type" TEXT,
    ADD COLUMN "media_title" TEXT,
    ADD COLUMN "media_poster_uri" TEXT,
    ADD COLUMN "media_release_date" DATE;

ALTER TABLE "reviews" RENAME COLUMN "title" TO "review_title";

ALTER TABLE "reviews" RENAME COLUMN "description" TO "review_description";

UPDATE
    "reviews" AS r
SET
    media_id = m.movie_id,
    imdb_id = m.imdb_id,
    media_type = 'movie',
    media_title = m.title,
    media_poster_uri = m.poster_path,
    media_release_date = m.release_date
FROM
    "movie_reviews" AS m
WHERE
    m.review_id = r.review_id;

UPDATE
    "reviews" AS r
SET
    media_id = s.show_id,
    imdb_id = s.imdb_id,
    media_type = 'show',
    media_title = s.name,
    media_poster_uri = s.poster_path,
    media_release_date = s.first_air_date
FROM
    "show_reviews" AS s
WHERE
    s.review_id = r.review_id;

ALTER TABLE "reviews"
    ALTER COLUMN "media_id" SET NOT NULL,
    ALTER COLUMN "media_type" SET NOT NULL,
    ALTER COLUMN "media_title" SET NOT NULL;

