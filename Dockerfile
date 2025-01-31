FROM rust:1.76-alpine as builder

# Install build dependencies first - this layer can be cached
RUN apk add --no-cache \
    musl-dev \
    openssl-dev \
    pkgconfig \
    build-base \
    cmake \
    git

WORKDIR /usr/src/app

# Create src directory and copy only files needed for dependency caching
COPY Cargo.toml Cargo.lock ./
RUN mkdir src && \
    touch src/lib.rs && \
    echo "fn main() {}" > src/main.rs

# Build and cache dependencies
RUN cargo build --release

# Copy actual source code
COPY . .

# Build the application
RUN cargo build --release

# Runtime stage
FROM alpine:latest

# Install runtime dependencies
RUN apk add --no-cache openssl

# Create necessary directories
RUN mkdir -p /etc/eth-indexer /data/eth-indexer && chmod 777 /data/eth-indexer

# Copy the binary and config
COPY --from=builder /usr/src/app/target/release/eth-high-perf-indexer /usr/local/bin/
COPY --from=builder /usr/src/app/config/default.toml /etc/eth-indexer/config.toml

# Set environment variable for config path
ENV CONFIG_PATH=/etc/eth-indexer/config.toml

# Expose metrics port
EXPOSE 9090

# Set the entrypoint
ENTRYPOINT ["eth-high-perf-indexer"]
