-- Your SQL goes here
CREATE TABLE "server_meta"(
    "key" text NOT NULL PRIMARY KEY,
    "value" jsonb DEFAULT '{}' NOT NULL
);

