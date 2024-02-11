-- Your SQL goes here
CREATE TABLE "users"(
	"user_id" UUID NOT NULL PRIMARY KEY,
	"first_name" TEXT NOT NULL,
	"last_name" TEXT NOT NULL,
	"email" TEXT NOT NULL,
	"avatar_uri" TEXT,
	"date_created" DATE NOT NULL,
	"permission_level" INT2 NOT NULL,
	"public" BOOL NOT NULL
);

CREATE TABLE "ratings"(
	"rating_id" UUID NOT NULL PRIMARY KEY,
	"user_id" UUID NOT NULL,
	"movie_id" INT4 NOT NULL,
	"movie_title" TEXT NOT NULL,
	"movie_poster_uri" TEXT NOT NULL,
	"movie_release_year" INT2 NOT NULL,
	"date" DATE NOT NULL,
	"value" INT2 NOT NULL,
	"review_title" TEXT,
	"review_description" TEXT,
	FOREIGN KEY ("user_id") REFERENCES "users"("user_id")
);
