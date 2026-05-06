-- posts에 저장한 모든 글을 불러온다.

SELECT id, title, LEDFT(body, 20) FROM posts ORDER BY id;

EXPLAIN ANALYZE
SELECT id FROM posts ORDER BY created_at DESC LIMIT 50 OFFSET 99950;

INSERT INTO posts (title, body, created_at, updated_at)
SELECT 'title ' || g, 'body ' || g, now() - (g * interval '1 second'), now()
FROM generate_series(1, 10000) g;

EXPLAIN ANALYZE
SELECT id, title, body, created_at, updated_at
FROM posts
ORDER BY created_at DESC, id DESC
LIMIT 20;

EXPLAIN ANALYZE
SELECT id, title, body, created_at, updated_at
FROM posts
WHERE (created_at, id) < ('2026-04-12 09:00:00+00', 9000)
ORDER BY created_at DESC, id DESC
LIMIT 20;

EXPLAIN ANALYZE
SELECT id, post_id, body, created_at, updated_at
FROM comments
WHERE post_id = 1
ORDER BY created_at DESC, id DESC
LIMIT 20;

SELECT email, LEFT(password_hash, 300) FROM users WHERE email = 'bob@example.com';