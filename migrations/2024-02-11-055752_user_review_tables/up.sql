-- Your SQL goes here
CREATE TABLE "users"(
	"user_id" UUID NOT NULL PRIMARY KEY,
	"email" TEXT NOT NULL,
	"password" TEXT NOT NULL,
	"first_name" TEXT NOT NULL,
	"last_name" TEXT NOT NULL,
	"avatar_uri" TEXT,
	"date_created" TIMESTAMP NOT NULL,
	"permission_level" INT2 NOT NULL,
	"public" BOOL NOT NULL
);

CREATE TABLE "reviews"(
	"review_id" UUID NOT NULL PRIMARY KEY,
	"user_id" UUID NOT NULL,
	"media_id" INT4 NOT NULL,
	"imdb_id" TEXT,
	"media_type" TEXT NOT NULL,
	"media_title" TEXT NOT NULL,
	"media_poster_uri" TEXT,
	"media_release_year" INT2 NOT NULL,
	"date" DATE,
	"rating" INT2 NOT NULL,
	"review_title" TEXT,
	"review_description" TEXT,
	"venue" TEXT,
	FOREIGN KEY ("user_id") REFERENCES "users"("user_id")
);

CREATE TABLE "review_company"(
	"review_id" UUID NOT NULL,
	"user_id" UUID NOT NULL,
	PRIMARY KEY ("review_id", "user_id"),
	FOREIGN KEY ("review_id") REFERENCES "reviews"("review_id"),
	FOREIGN KEY ("user_id") REFERENCES "users"("user_id")
);
