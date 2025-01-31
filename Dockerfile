# Stage 1: Builder
FROM rust:1.76-alpine as builder

# Install necessary build dependencies
RUN apk add --no-cache \
    musl-dev \
    openssl-dev \
    pkgconfig \
    build-base \
    cmake \
    git

WORKDIR /usr/src/app

# Copy only Cargo.toml and Cargo.lock first
COPY Cargo.toml Cargo.lock ./

# Create a dummy main.rs to pre-build dependencies
RUN mkdir src && \
    echo "fn main() {}" > src/main.rs && \
    cargo build --release && \
    rm -rf src/

# Now copy the actual source code
COPY . .

# Build the application
RUN cargo build --release

# Stage 2: Runtime
FROM alpine:latest

# Install runtime dependencies
RUN apk add --no-cache \
    openssl

# Create necessary directories
RUN mkdir -p /etc/eth-indexer /data/eth-indexer && \
    chmod 777 /data/eth-indexer

# Copy the binary and config from the builder stage
COPY --from=builder /usr/src/app/target/release/eth-high-perf-indexer /usr/local/bin/
COPY --from=builder /usr/src/app/config/default.toml /etc/eth-indexer/config.toml

# Set environment variable for config path
ENV CONFIG_PATH=/etc/eth-indexer/config.toml

# Expose metrics port
EXPOSE 9090

# Set the entrypoint
ENTRYPOINT ["eth-high-perf-indexer"]
