FROM rust:1.76-alpine as builder
WORKDIR /usr/src/app

# Install build dependencies
RUN apk add --no-cache musl-dev

# Copy the source code
COPY . .

# Remove existing Cargo.lock and build
RUN rm -f Cargo.lock && cargo build --release

FROM alpine:latest

# Create necessary directories
RUN mkdir -p /etc/eth-indexer /data/eth-indexer && \
    chmod 777 /data/eth-indexer

# Copy the binary and config
COPY --from=builder /usr/src/app/target/release/eth-high-perf-indexer /usr/local/bin/
COPY --from=builder /usr/src/app/config/default.toml /etc/eth-indexer/config.toml

# Set environment variable for config path
ENV CONFIG_PATH=/etc/eth-indexer/config.toml

# Expose metrics port
EXPOSE 9090

# Set the entrypoint
ENTRYPOINT ["eth-high-perf-indexer"]
