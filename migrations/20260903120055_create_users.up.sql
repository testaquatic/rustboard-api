-- Add up migration script here
CREATE TABLE users(
  id bigint GENERATED ALWAYS AS IDENTITY PRIMARY KEY,
  email varchar(255) NOT NULL UNIQUE,
  password_hash varchar(255) NOT NULL,
  display_name varchar(100) NOT NULL,
  role VARCHAR(20) NOT NULL DEFAULT 'user',
  created_at timestamp with time zone NOT NULL DEFAULT NOW(),
  updated_at timestamp with time zone NOT NULL DEFAULT NOW()
);

