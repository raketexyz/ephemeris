CREATE EXTENSION IF NOT EXISTS "uuid-ossp";

CREATE TABLE "comments" (
    "id" SERIAL PRIMARY KEY,
    "author" TEXT NOT NULL REFERENCES "users" ("username") ON DELETE CASCADE ON UPDATE CASCADE,
    "post" INT NOT NULL REFERENCES "posts" ("id") ON DELETE CASCADE,
    "message" TEXT NOT NULL,
    "created_at" TIMESTAMP NOT NULL DEFAULT current_timestamp,
    "updated_at" TIMESTAMP
);

SELECT diesel_manage_updated_at('comments');
