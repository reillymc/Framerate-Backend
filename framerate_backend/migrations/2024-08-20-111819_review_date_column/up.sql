-- Your SQL goes here
ALTER TABLE "reviews"
ADD COLUMN "media_release_date" DATE;

-- transfer data from media_release_year to media_release_date
UPDATE "reviews"
SET "media_release_date" = DATE_TRUNC(
        'year',
        TO_TIMESTAMP("media_release_year" || '-01-01', 'YYYY-MM-DD')
    );

-- -- drop media_release_year column
ALTER TABLE "reviews" DROP COLUMN "media_release_year";
