FROM lukemathwalker/cargo-chef:latest-rust-latest AS chef
WORKDIR /app

FROM chef AS planner
COPY . .
RUN cargo chef prepare --recipe-path recipe.json

FROM chef AS builder
COPY --from=planner /app/recipe.json recipe.json
RUN cargo chef cook --release --recipe-path recipe.json
COPY . .
RUN cargo build --release --bin app

FROM debian:bookworm-slim AS runtime
COPY --from=builder /app/target/release/app /usr/local/bin/app
ARG PORT=8081
ENV PORT=${PORT}
EXPOSE ${PORT}
CMD ["sh", "-c", "PORT=${PORT} /usr/local/bin/app"]
CMD ["app"]
