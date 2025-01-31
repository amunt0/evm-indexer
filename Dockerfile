FROM rust:1.76-alpine as builder
RUN apk add --no-cache \
    musl-dev \
    openssl-dev \
    pkgconfig \
    build-base \
    libressl-dev \
    cmake \
    git
WORKDIR /usr/src/app
COPY Cargo.toml Cargo.lock ./
RUN mkdir src && echo "fn main() {}" > src/main.rs && cargo build --release && rm -rf src/
COPY . .
RUN cargo build --release

FROM alpine:latest
RUN apk add --no-cache openssl libressl
WORKDIR /usr/local/bin
COPY --from=builder /usr/src/app/target/release/eth-high-perf-indexer .
COPY --from=builder /usr/src/app/config/default.toml /etc/eth-indexer/config.toml
ENV CONFIG_PATH=/etc/eth-indexer/config.toml
EXPOSE 9090
ENTRYPOINT ["eth-high-perf-indexer"]
