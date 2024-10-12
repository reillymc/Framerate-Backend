-- Your SQL goes here
CREATE TABLE "season_reviews"(
    "review_id" uuid NOT NULL PRIMARY KEY,
    "user_id" uuid NOT NULL,
    "show_id" int4 NOT NULL,
    "season_number" int4 NOT NULL,
    "name" text,
    "poster_path" text,
    "air_date" date,
    FOREIGN KEY ("review_id") REFERENCES "reviews"("review_id"),
    FOREIGN KEY ("user_id") REFERENCES "users"("user_id")
);

