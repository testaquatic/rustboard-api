-- Add up migration script here
CREATE TABLE comments(
  id bigint GENERATED ALWAYS AS IDENTITY PRIMARY KEY,
  post_id bigint NOT NULL REFERENCES posts(id) ON DELETE CASCADE,
  content text NOT NULL,
  author_id bigint NOT NULL REFERENCES users(id) ON DELETE CASCADE,
  created_at timestamp with time zone NOT NULL DEFAULT NOW(),
  updated_at timestamp with time zone NOT NULL DEFAULT NOW()
);

CREATE INDEX comments_post_id_idx ON comments(post_id, created_at DESC);

