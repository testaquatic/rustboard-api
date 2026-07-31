# Rust 웹 백엔드(Axum)

[Rust 웹 백엔드(Axum)](https://text.ibetter.kr/rust-axum)을 읽으면서 작성한 코드이다.

문서 주소 : [https://text.ibetter.kr/rust-axum](https://text.ibetter.kr/rust-axum)

# 실행 환경

## POSTGRES

### 도커 이미지 생성

```bash
docker run -d \
  --name rustboard-db \
  -p 5432:5432 \
  -e POSTGRES_PASSWORD=postgres \
  -e POSTGRES_DB=rustboard \
  -v rustboard-db-data:/var/lib/postgresql/18/docker \
  postgres:18
```

### 실행 확인

```bash
docker exec -it rustboard-db psql -U postgres -d rustboard -c "SELECT version();"
```

### 중지

```bash
docker stop rustboard-db
```

### 시작

```bash
docker start rustboard-db
```

### 로그보기

```bash
docker logs -f rustboard-db
```

# API

[Swagger UI](http://localhost:3000/swagger-ui/)
