# Rust 웹 백엔드(Axum)

[Rust 웹 백엔드(Axum)](https://text.ibetter.kr/rust-axum)을 읽으면서 작성한 코드이다.

문서 주소 : [https://text.ibetter.kr/rust-axum](https://text.ibetter.kr/rust-axum)

# 실행 환경

1. 도커
```bash
docker run -d \
  --name rustboard-pg \
  -e POSTGRES_USER=rustboard \
  -e POSTGRES_PASSWORD=rustboard \
  -e POSTGRES_DB=rustboard \
  -p 5432:5432 \
  -v rustboard-pgdata:/var/lib/postgresql/data \
  postgres:16
```

2. PostgreSQL
  - SQLX
```bash
cargo sqlx database create
cargo sqlx migrate info
```

# 엔드포인트
/swagger 를 확인한다.