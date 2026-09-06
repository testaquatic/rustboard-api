# Rust 웹 백엔드(Axum)

[Rust 웹 백엔드(Axum)](https://text.ibetter.kr/rust-axum)을 읽으면서 작성한 코드이다.

문서 주소 : [https://text.ibetter.kr/rust-axum](https://text.ibetter.kr/rust-axum)

# 실행환경 설정

## Postgresql

1. 도커 이미지 생성과 실행

```bash
docker run -d \
  --name rustboard-db \
  -p 5432:5432 \
  -e POSTGRES_PASSWORD=postgres \
  -e POSTGRES_DB=rustboard \
  -v rustboard-db-data:/var/lib/postgresql/18/docker \
  postgres:18 \
  -c max_connections=200
```

2. 실행 확인

```bash
docker exec -it rustboard-db psql -U postgres -d rustboard -c "SELECT version();"
```

3. 시작

```
docker start rustboard-db
```

4. 중지

```bash
docker stop rustboard-db
```

5. 로그 보기

```bash
docker logs -f rustboard-db
```
