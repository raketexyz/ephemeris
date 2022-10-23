CREATE EXTENSION IF NOT EXISTS "uuid-ossp";

CREATE TABLE "tokens" (
    "id" UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    "user" UUID NOT NULL REFERENCES "users" ON DELETE CASCADE,
    "expiration" TIMESTAMP NOT NULL,
    "created_at" TIMESTAMP NOT NULL DEFAULT current_timestamp,
    "updated_at" TIMESTAMP
);

SELECT diesel_manage_updated_at('tokens');
