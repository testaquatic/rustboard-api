# Rust 웹 백엔드(Axum)

[Rust 웹 백엔드(Axum)](https://text.ibetter.kr/rust-axum)을 읽으면서 작성한 코드이다.

문서 주소 : [https://text.ibetter.kr/rust-axum](https://text.ibetter.kr/rust-axum)

# 실행 환경

1. 도커

```bash
docker run -d \
  --name rustboard-db \
  -p 5432:5432 \
  -e POSTGRES_PASSWORD=postgres \
  -e POSTGRES_DB=rustboard \
  -v rustboard-db-data:/var/lib/postgresql/data \
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
