-- Your SQL goes here
ALTER TABLE "watchlists" RENAME COLUMN "watchlist_id" TO "collection_id";

ALTER TABLE "watchlists" RENAME TO "collections";

ALTER TABLE "show_entries" RENAME COLUMN "watchlist_id" TO "collection_id";

ALTER TABLE "movie_entries" RENAME COLUMN "watchlist_id" TO "collection_id";

ALTER TABLE movie_entries
    DROP CONSTRAINT movie_entries_watchlist_id_fkey,
    ADD CONSTRAINT movie_entries_collection_id_fkey FOREIGN KEY (collection_id) REFERENCES collections(collection_id) ON DELETE CASCADE;

ALTER TABLE show_entries
    DROP CONSTRAINT show_entries_watchlist_id_fkey,
    ADD CONSTRAINT show_entries_collection_id_fkey FOREIGN KEY (collection_id) REFERENCES collections(collection_id) ON DELETE CASCADE;

