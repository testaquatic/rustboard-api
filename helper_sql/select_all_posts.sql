-- posts에 저장한 모든 글을 불러온다.

SELECT id, title, left(body, 20) FROM posts ORDER BY id;