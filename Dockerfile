FROM rust:slim AS chef-planner
WORKDIR /app
RUN cargo install cargo-chef
COPY . .
RUN cargo chef prepare --recipe-path recipe.json

FROM rust:slim AS chef-builder
WORKDIR /app
RUN cargo install cargo-chef
RUN apt update && apt install curl -y
COPY --from=chef-planner /app/recipe.json recipe.json
RUN cargo chef cook --release --recipe-path recipe.json
COPY . .
COPY .sqlx/ .sqlx/
ENV SQLX_OFFLINE=true
RUN cargo build --release

FROM gcr.io/distroless/cc-debian13 AS runtime
WORKDIR /app
COPY --from=chef-builder /app/target/release/rustboard-api /app/rustboard-api
COPY ./configuration/base.yaml /app/configuration/base.yaml
EXPOSE 3000
ENTRYPOINT [ "/app/rustboard-api" ]