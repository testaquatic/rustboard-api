-- Add down migration script here
DROP INDEX IF EXISTS posts_created_at_idx;
DROP TABLE IF EXISTS posts;
