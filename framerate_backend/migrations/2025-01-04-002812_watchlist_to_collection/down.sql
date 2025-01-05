-- This file should undo anything in `up.sql`
ALTER TABLE "collections" RENAME TO "watchlists";

ALTER TABLE "watchlists" RENAME COLUMN "collection_id" TO "watchlist_id";

ALTER TABLE "show_entries" RENAME COLUMN "collection_id" TO "watchlist_id";

ALTER TABLE "movie_entries" RENAME COLUMN "collection_id" TO "watchlist_id";

ALTER TABLE movie_entries
    DROP CONSTRAINT movie_entries_collection_id_fkey,
    ADD CONSTRAINT movie_entries_watchlist_id_fkey FOREIGN KEY (collection_id) REFERENCES collections(collection_id);

ALTER TABLE show_entries
    DROP CONSTRAINT show_entries_collection_id_fkey,
    ADD CONSTRAINT show_entries_watchlist_id_fkey FOREIGN KEY (collection_id) REFERENCES collections(collection_id);

