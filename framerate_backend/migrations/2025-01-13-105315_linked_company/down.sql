-- This file should undo anything in `up.sql`
ALTER TABLE users
    ALTER COLUMN email DROP NOT NULL;

ALTER TABLE users
    ALTER COLUMN PASSWORD DROP NOT NULL;

INSERT INTO "users"(user_id, first_name, last_name, date_created, permission_level, public, created_by)
SELECT
    company_id,
    first_name,
    last_name,
    date_created,
    -20,
    FALSE,
    created_by
FROM
    company;

ALTER TABLE "review_company" RENAME COLUMN "company_id" TO "user_id";

ALTER TABLE "review_company"
    DROP CONSTRAINT review_company_company_id_fkey,
    ADD CONSTRAINT review_company_user_id_fkey FOREIGN KEY (user_id) REFERENCES users(user_id) ON DELETE CASCADE;

DROP TABLE IF EXISTS "company";

