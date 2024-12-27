-- Your SQL goes here
ALTER TABLE "users"
ALTER COLUMN "email" DROP NOT NULL;

ALTER TABLE "users"
ALTER COLUMN "password" DROP NOT NULL;

ALTER TABLE "users"
ADD CONSTRAINT "users_email_unique" UNIQUE NULLS DISTINCT ("email");
