-- This file should undo anything in `up.sql`
ALTER TABLE "reviews"
ADD COLUMN "media_release_year" INT2;

UPDATE "reviews"
SET "media_release_year" = DATE_PART('year', "media_release_date");

ALTER TABLE "reviews"
ALTER COLUMN "media_release_year"
SET NOT NULL;

ALTER TABLE "reviews" DROP COLUMN "media_release_date";
