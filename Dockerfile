# ビルドステージ
FROM rust:1.83-bookworm AS builder
WORKDIR /usr/src/mimo-server
COPY Cargo.toml ./
# 依存関係をキャッシュするために仮ビルドを実施
RUN mkdir src && \
    echo "fn main() {}" > src/main.rs && \
    cargo build --release && \
    rm -rf src && \
    rm -f target/release/deps/mimo_server*
COPY src ./src
RUN cargo build --release

# プロダクションステージ
FROM debian:bookworm-slim
WORKDIR /app
RUN apt-get update && \
    apt-get install -y --no-install-recommends \
    ca-certificates \
    tzdata && \
    rm -rf /var/lib/apt/lists/*
RUN groupadd -r mimo-group && useradd -r -g mimo-group mimo-user
USER mimo-user:mimo-group
COPY --from=builder /usr/src/mimo-server/target/release/mimo-server /app/mimo-server
EXPOSE 5050
CMD ["/app/mimo-server"]
