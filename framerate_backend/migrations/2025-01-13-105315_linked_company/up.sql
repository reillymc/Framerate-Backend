-- Your SQL goes here
CREATE TABLE "company"(
    "company_id" uuid NOT NULL PRIMARY KEY,
    "first_name" text NOT NULL,
    "last_name" text NOT NULL,
    "date_created" timestamp NOT NULL,
    "created_by" uuid NOT NULL,
    "user_id" uuid,
    FOREIGN KEY ("created_by") REFERENCES "users"("user_id"),
    FOREIGN KEY ("user_id") REFERENCES "users"("user_id")
);

INSERT INTO "company"(company_id, first_name, last_name, date_created, created_by)
SELECT
    user_id,
    first_name,
    last_name,
    date_created,
    created_by
FROM
    users
WHERE
    permission_level = 0
    OR permission_level = - 20
    OR email IS NULL
    OR PASSWORD IS NULL;

ALTER TABLE "review_company" RENAME COLUMN "user_id" TO "company_id";

ALTER TABLE "review_company"
    DROP CONSTRAINT review_company_user_id_fkey,
    ADD CONSTRAINT review_company_company_id_fkey FOREIGN KEY (company_id) REFERENCES company(company_id) ON DELETE CASCADE;

DELETE FROM users
WHERE permission_level = 0
    OR permission_level = - 20
    OR email IS NULL
    OR PASSWORD IS NULL;

ALTER TABLE users
    ALTER COLUMN email SET NOT NULL;

ALTER TABLE users
    ALTER COLUMN PASSWORD SET NOT NULL;

