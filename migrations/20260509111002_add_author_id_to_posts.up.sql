-- Add up migration script here
ALTER TABLE posts ADD COLUMN author_id BIGINT REFERENCES users(id);

UPDATE posts SET author_id = 1 WHERE author_id IS NULL;

ALTER TABLE posts ALTER COLUMN author_id SET NOT NULL;

CREATE INDEX idx_posts_author_id ON posts(author_id);