-- posts 테이블에 author_id 컬럼 추가.
ALTER TABLE posts ADD COLUMN author_id BIGINT REFERENCES users(id);

-- 기존 행에 기본 사용자 할당
UPDATE posts SET author_id = 1 WHERE author_id IS NULL;

-- NOT NULL 제약조건 추가
ALTER TABLE posts ALTER COLUMN author_id SET NOT NULL;

-- 인덱스 추가
CREATE INDEX idx_posts_author_id ON posts(author_id);
