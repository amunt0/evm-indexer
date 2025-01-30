# Stage 1: Build the application
FROM rust:1.76-alpine as builder

# Set the working directory
WORKDIR /usr/src/app

# Install build dependencies including musl-dev, pkg-config, and openssl-dev
RUN apk add --no-cache \
    musl-dev \
    pkgconfig \
    openssl-dev \
    build-base

# Copy Cargo.toml and Cargo.lock first to leverage Docker's layer caching
COPY Cargo.toml Cargo.lock ./

# Create dummy main.rs to avoid re-running cargo build on non-code changes
RUN mkdir -p src && echo "fn main() {}" > src/main.rs

# Download dependencies (this step will be cached if Cargo.toml and Cargo.lock don't change)
RUN cargo build --release --locked

# Remove the dummy file
RUN rm -f src/main.rs

# Copy the actual source code into the container
COPY . .

# Rebuild the project with the actual source code
RUN rm -f target/release/deps/eth_high_perf_indexer* && \
    RUST_BACKTRACE=1 cargo build --release --verbose

# Stage 2: Create the final image
FROM alpine:latest

# Install runtime dependencies for OpenSSL
RUN apk add --no-cache \
    openssl

# Create necessary directories
RUN mkdir -p /etc/eth-indexer /data/eth-indexer && \
    chmod 777 /data/eth-indexer

# Copy the binary from the builder stage
COPY --from=builder /usr/src/app/target/release/eth-high-perf-indexer /usr/local/bin/

# Copy the configuration file
COPY --from=builder /usr/src/app/config/default.toml /etc/eth-indexer/config.toml

# Set environment variable for config path
ENV CONFIG_PATH=/etc/eth-indexer/config.toml

# Expose metrics port
EXPOSE 9090

# Set the entrypoint
ENTRYPOINT ["eth-high-perf-indexer"]
