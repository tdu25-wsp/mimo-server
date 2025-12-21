# ビルドステージ
FROM rust:alpine AS builder
RUN apk add --no-cache musl-dev openssl-dev pkgconfig
WORKDIR /usr/src/mimo-server
COPY Cargo.toml ./
# 依存関係をキャッシュするために仮ビルドを実施
RUN mkdir src && \
    echo "fn main() {}" > src/main.rs && \
    cargo build --release && \
    rm -rf src && \
    rm -f target/release/deps/mimo_server*
COPY src ./src
COPY migrations ./migrations
RUN cargo build --release

# プロダクションステージ
FROM alpine:latest
WORKDIR /app
RUN apk add --no-cache openssl ca-certificates tzdata
RUN addgroup -S mimo-group && adduser -S mimo-user -G mimo-group
USER mimo-user:mimo-group
COPY --from=builder /usr/src/mimo-server/target/release/mimo-server /app/mimo-server
EXPOSE 5050
CMD ["/app/mimo-server"]
