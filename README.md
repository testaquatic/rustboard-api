# Rust 웹 백엔드(Axum)

[Rust 웹 백엔드(Axum)](https://text.ibetter.kr/rust-axum)을 읽으면서 작성한 코드이다.  
문서 주소 : [https://text.ibetter.kr/rust-axum](https://text.ibetter.kr/rust-axum)

# 실행 환경

## POSTGRES

- 도커 이미지 생성

```bash
docker run -d \
  --name rustboard-db \
  -p 5432:5432 \
  -e POSTGRES_PASSWORD=postgres \
  -e POSTGRES_DB=rustboard \
  -v rustboard-db-data:/var/lib/postgresql/data \
  postgres:16
```

- SQLX를 이용한 마이그레이션

```bash
cargo sqlx database create
cargo sqlx migrate info
```

# 엔드포인트

/swagger 를 확인한다.
