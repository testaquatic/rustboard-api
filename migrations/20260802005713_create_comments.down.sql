-- Add down migration script here
DROP INDEX IF EXISTS comments_post_id_idx;
DROP TABLE IF EXISTS comments;