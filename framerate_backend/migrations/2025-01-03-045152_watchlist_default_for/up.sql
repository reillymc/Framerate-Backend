-- Your SQL goes here
ALTER TABLE "watchlists"
    ADD COLUMN "default_for" TEXT;

UPDATE
    "watchlists" AS w
SET
    default_for = 'watchlist';

