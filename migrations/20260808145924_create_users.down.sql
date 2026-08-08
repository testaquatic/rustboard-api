-- Add down migration script here
DROP INDEX IF EXISTS idx_users_email;
DROP TABLE IF EXISTS users;