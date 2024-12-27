-- Your SQL goes here
ALTER TABLE review_company
    DROP CONSTRAINT review_company_user_id_fkey,
    ADD CONSTRAINT review_company_user_id_fkey FOREIGN KEY (user_id) REFERENCES users(user_id) ON DELETE CASCADE;

