# Planner
FROM rust:slim-trixie AS chef-planner
WORKDIR /app
RUN cargo install cargo-chef
COPY . .
RUN cargo chef prepare --recipe-path recipe.json

# Builder
FROM rust:slim AS chef-builder
WORKDIR /app
RUN cargo install cargo-chef
## 필요한 의존성 설치
RUN apt update && apt install curl -y

## 의존성 캐시 레이어
COPY --from=chef-planner /app/recipe.json recipe.json
RUN cargo chef cook --release --recipe-path recipe.json

## 소스 복사 및 빌드
COPY . .

## sqlx 오프라인 모드
COPY .sqlx/ .sqlx/
ENV SQLX_OFFLINE=true


RUN cargo build --release

# 런타임
FROM gcr.io/distroless/cc-debian13:nonroot AS runtime
WORKDIR /app

COPY --from=chef-builder /app/target/release/rustboard-api /app/rustboard-api

EXPOSE 3000
ENTRYPOINT [ "/app/rustboard-api" ]