FROM rust:1.76-alpine as builder
RUN apk add --no-cache \
    musl-dev \
    openssl-dev \
    openssl-libs-static \
    pkgconfig \
    build-base \
    cmake \
    git
ENV OPENSSL_STATIC=1
ENV OPENSSL_LIB_DIR=/usr/lib
ENV OPENSSL_INCLUDE_DIR=/usr/include/openssl
WORKDIR /usr/src/app
# Copy only the dependency files first
COPY Cargo.toml Cargo.lock ./
# Create a dummy main.rs to build dependencies
RUN mkdir src && echo "fn main() {}" > src/main.rs
# Pre-build dependencies with features
RUN cargo build --release --features tracing-subscriber/chrono && rm -rf src/
# Now copy the actual source code
COPY . .
# Build the application 
RUN cargo build --release --features tracing-subscriber/chrono
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
