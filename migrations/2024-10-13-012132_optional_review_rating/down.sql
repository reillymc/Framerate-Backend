-- This file should undo anything in `up.sql`
ALTER TABLE "reviews"
    ALTER COLUMN "rating" SET NOT NULL,
