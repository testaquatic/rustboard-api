INSERT INTO posts (title, body, created_at, updated_at)
SELECT 'title ' || g, 'body' || g, now() - (g * interval '1 seconds'), now()
FROM generate_series(1, 10000) g;

EXPLAIN ANALYZE
SELECT id, title, body, created_at, updated_at
FROM posts
ORDER BY created_at DESC, id DESC
LIMIT 20;

EXPLAIN ANALYZE
SELECT id, title, body, created_at, updated_at
FROM posts
WHERE (created_at, id) < (now(), 9000)
ORDER BY created_at DESC, id DESC
LIMIT 20;

EXPLAIN ANALYZE
SELECT id, post_id, body, created_at, updated_at
FROM comments
WHERE post_id = 5
ORDER BY created_at DESC, id DESC
LIMIT 20;