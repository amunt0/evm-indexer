FROM rust:1.76-slim-bullseye as builder

RUN apt-get update && apt-get install -y \
    pkg-config libssl-dev build-essential cmake \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /usr/src/app
COPY Cargo.toml ./
RUN mkdir src && echo "fn main() {}" > src/main.rs
RUN cargo init && cargo build --release
RUN rm -rf src/

COPY . .
RUN cargo build --release

FROM debian:bullseye-slim
RUN apt-get update && apt-get install -y \
    libssl1.1 ca-certificates \
    && rm -rf /var/lib/apt/lists/*

COPY --from=builder /usr/src/app/target/release/eth-high-perf-indexer /usr/local/bin/
COPY --from=builder /usr/src/app/config/default.toml /etc/eth-indexer/config.toml

RUN mkdir -p /data/eth-indexer && chmod 777 /data/eth-indexer
ENV CONFIG_PATH=/etc/eth-indexer/config.toml
EXPOSE 9090
ENTRYPOINT ["eth-high-perf-indexer"]
