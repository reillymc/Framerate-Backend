-- Your SQL goes here
ALTER TABLE "users"
ADD COLUMN "configuration" JSONB DEFAULT '{}' NOT NULL;
