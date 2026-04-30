-- Add up migration script here
CREATE TABLE posts (
  id BIGINT GENERATED ALWAYS AS IDENTITY PRIMARY KEY,
  title TEXT NOT NULL,
  body TEXT NOT NULL,
  created_AT TIMESTAMPTZ NOT NULL DEFAULT now(),
  updated_AT TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE INDEX posts_created_at_idx ON posts(created_at DESC);