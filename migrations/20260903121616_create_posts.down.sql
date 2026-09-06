-- Add down migration script here
DROP INDEX IF EXISTS idx_posts_author_id;

DROP INDEX IF EXISTS idx_posts_created_at;

DROP TABLE IF EXISTS posts;

