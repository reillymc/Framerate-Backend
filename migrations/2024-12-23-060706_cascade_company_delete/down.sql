-- This file should undo anything in `up.sql`
ALTER TABLE review_company
    DROP CONSTRAINT review_company_user_id_fkey,
    ADD CONSTRAINT review_company_user_id_fkey FOREIGN KEY (user_id) REFERENCES users(user_id);

