# Build stage
FROM rust:1.76-alpine as builder

# Install build dependencies - this layer can be cached
RUN apk add --no-cache \
    musl-dev \
    openssl-dev \
    pkgconfig \
    build-base \
    cmake \
    git

# Set build environment variables for optimization
ENV RUSTFLAGS="-C target-feature=+crt-static"
ENV CARGO_PROFILE_RELEASE_LTO="true"
ENV CARGO_PROFILE_RELEASE_CODEGEN_UNITS="1"
ENV CARGO_NET_GIT_FETCH_WITH_CLI="true"

WORKDIR /usr/src/app

# Copy only the dependency files first
COPY Cargo.toml Cargo.lock ./

# Create a dummy main.rs to build dependencies
RUN mkdir src && \
    echo "fn main() {println!(\"dummy\");}" > src/main.rs && \
    # Build dependencies only
    cargo build --release && \
    # Remove the dummy source
    rm -rf src/ target/release/deps/eth_high_perf_indexer*

# Copy the actual source code
COPY src/ src/
COPY config/ config/

# Build the final binary
RUN cargo build --release && \
    strip target/release/eth-high-perf-indexer

# Runtime stage - using a specific version instead of latest
FROM alpine:3.19

# Install runtime dependencies and create directories in one layer
RUN apk add --no-cache openssl ca-certificates && \
    mkdir -p /etc/eth-indexer /data/eth-indexer && \
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
