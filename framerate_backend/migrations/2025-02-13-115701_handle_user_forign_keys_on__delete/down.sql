-- This file should undo anything in `up.sql`
ALTER TABLE "collections"
    DROP CONSTRAINT collections_user_id_fkey,
    ADD CONSTRAINT watchlists_user_id_fkey FOREIGN KEY (user_id) REFERENCES users(user_id);

ALTER TABLE "company"
    DROP CONSTRAINT company_created_by_fkey,
    ADD CONSTRAINT company_created_by_fkey FOREIGN KEY (user_id) REFERENCES users(user_id);

ALTER TABLE "company"
    DROP CONSTRAINT company_user_id_fkey,
    ADD CONSTRAINT company_user_id_fkey FOREIGN KEY (user_id) REFERENCES users(user_id);

ALTER TABLE "movie_entries"
    DROP CONSTRAINT movie_entries_user_id_fkey,
    ADD CONSTRAINT movie_entries_user_id_fkey FOREIGN KEY (user_id) REFERENCES users(user_id);

ALTER TABLE "movie_reviews"
    DROP CONSTRAINT movie_reviews_user_id_fkey,
    ADD CONSTRAINT movie_reviews_user_id_fkey FOREIGN KEY (user_id) REFERENCES users(user_id);

ALTER TABLE "review_company"
    DROP CONSTRAINT review_company_company_id_fkey,
    ADD CONSTRAINT review_company_company_id_fkey FOREIGN KEY (company_id) REFERENCES company(company_id);

ALTER TABLE "review_company"
    DROP CONSTRAINT review_company_review_id_fkey,
    ADD CONSTRAINT review_company_review_id_fkey FOREIGN KEY (review_id) REFERENCES reviews(review_id);

ALTER TABLE "reviews"
    DROP CONSTRAINT reviews_user_id_fkey,
    ADD CONSTRAINT reviews_user_id_fkey FOREIGN KEY (user_id) REFERENCES users(user_id);

ALTER TABLE "season_reviews"
    DROP CONSTRAINT season_reviews_user_id_fkey,
    ADD CONSTRAINT season_reviews_user_id_fkey FOREIGN KEY (user_id) REFERENCES users(user_id);

ALTER TABLE "show_entries"
    DROP CONSTRAINT show_entries_user_id_fkey,
    ADD CONSTRAINT show_entries_user_id_fkey FOREIGN KEY (user_id) REFERENCES users(user_id);

ALTER TABLE "show_reviews"
    DROP CONSTRAINT show_reviews_user_id_fkey,
    ADD CONSTRAINT show_reviews_user_id_fkey FOREIGN KEY (user_id) REFERENCES users(user_id);

