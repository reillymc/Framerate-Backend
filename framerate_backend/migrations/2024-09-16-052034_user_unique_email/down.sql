-- This file should undo anything in `up.sql`
ALTER TABLE "users"
ALTER COLUMN "email"
SET NOT NULL;

ALTER TABLE "users"
ALTER COLUMN "password"
SET NOT NULL;

ALTER TABLE "users" DROP CONSTRAINT "users_email_unique";
