
# 1. Build stage
FROM rust:1.96-bookworm AS builder

WORKDIR /app

# Copy manifests first.
# This allows Docker to cache dependency compilation.
COPY Cargo.toml Cargo.lock ./

# Create a dummy project to cache dependencies.
RUN mkdir src && \
    echo "fn main() {}" > src/main.rs && \
    cargo build --release && \
    rm -rf src

# Copy the actual source code and migrations.
COPY src ./src
COPY migrations ./migrations
COPY .sqlx ./.sqlx

# Build FluxPay
ENV SQLX_OFFLINE=true
RUN cargo build --release


# 2. Runtime stage
FROM debian:bookworm-slim AS runtime

WORKDIR /app

# Runtime dependencies required by Rust binaries
RUN apt-get update && \
    apt-get install -y --no-install-recommends \
        ca-certificates && \
    rm -rf /var/lib/apt/lists/*

# Copy the compiled FluxPay binary
COPY --from=builder /app/target/release/flux-pay /app/fluxpay

# Copy SQLx migrations
COPY --from=builder /app/migrations /app/migrations

# Application port
EXPOSE 8080

# Run FluxPay
CMD ["/app/fluxpay"]