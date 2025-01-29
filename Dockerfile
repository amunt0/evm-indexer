FROM rust:1.76-alpine as builder

WORKDIR /usr/src/app

# Install build dependencies
RUN apk add --no-cache musl-dev

# First copy only the Cargo.toml files
COPY Cargo.toml ./

# Create a dummy main.rs to build dependencies
RUN mkdir src && \
    echo "fn main() {}" > src/main.rs && \
    cargo update && \
    cargo build --release && \
    rm -f target/release/deps/eth_high_perf_indexer*

# Now copy the real source code
COPY . .

# Clean any existing Cargo.lock and rebuild
RUN rm -f Cargo.lock && \
    cargo update && \
    cargo build --release

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
