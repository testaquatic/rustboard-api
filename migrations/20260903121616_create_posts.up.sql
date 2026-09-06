-- Add up migration script here
CREATE TABLE posts(
  id bigint GENERATED ALWAYS AS IDENTITY PRIMARY KEY,
  title varchar(255) NOT NULL,
  content text NOT NULL,
  author_id bigint NOT NULL REFERENCES users(id),
  created_at timestamp with time zone NOT NULL DEFAULT NOW(),
  updated_at timestamp with time zone NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_posts_author_id ON posts(author_id);

CREATE INDEX idx_posts_created_at ON posts(created_at DESC);

