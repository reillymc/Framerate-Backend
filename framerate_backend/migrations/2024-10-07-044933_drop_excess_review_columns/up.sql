-- Your SQL goes here
ALTER TABLE "reviews"
    DROP COLUMN "media_id",
    DROP COLUMN "imdb_id",
    DROP COLUMN "media_type",
    DROP COLUMN "media_title",
    DROP COLUMN "media_poster_uri",
    DROP COLUMN "media_release_date";

ALTER TABLE "reviews" RENAME COLUMN "review_title" TO "title";

ALTER TABLE "reviews" RENAME COLUMN "review_description" TO "description";

