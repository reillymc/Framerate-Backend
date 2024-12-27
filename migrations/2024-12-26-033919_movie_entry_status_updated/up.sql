-- Your SQL goes here
ALTER TABLE movie_entries
    ADD COLUMN "status" TEXT,
    ADD COLUMN "updated_at" date NOT NULL DEFAULT CURRENT_DATE;

