-- This file should undo anything in `up.sql`
ALTER TABLE movie_entries
    DROP COLUMN "status",
    DROP COLUMN "updated_at";

