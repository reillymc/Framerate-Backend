-- Your SQL goes here
ALTER TABLE "users"
    ADD COLUMN "created_by" UUID;

ALTER TABLE "users"
    ADD CONSTRAINT fk_users_created_by FOREIGN KEY (created_by) REFERENCES users(user_id);

