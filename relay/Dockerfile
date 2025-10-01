# Multi-stage build for Sova Relay
FROM rust:1.75-slim as builder

# Install system dependencies
RUN apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev \
    && rm -rf /var/lib/apt/lists/*

# Set working directory
WORKDIR /app

# Copy workspace files first for dependency caching
COPY Cargo.toml ./
COPY relay/Cargo.toml ./relay/
COPY relay/src/ ./relay/src/

# Optimize for low-memory builds
ENV CARGO_BUILD_JOBS=1
ENV CARGO_TARGET_DIR=/tmp/target

# Build the relay server (build from workspace root)
RUN cargo build --release -p sova-relay

# Runtime stage
FROM debian:bookworm-slim

# Install runtime dependencies
RUN apt-get update && apt-get install -y \
    ca-certificates \
    && rm -rf /var/lib/apt/lists/*

# Create non-root user
RUN useradd -r -s /bin/false relay

# Copy binary from builder
COPY --from=builder /tmp/target/release/sova-relay /usr/local/bin/

# Set ownership
RUN chown relay:relay /usr/local/bin/sova-relay

# Switch to non-root user
USER relay

# Expose ports
EXPOSE 9090 8080

# Health check
HEALTHCHECK --interval=30s --timeout=10s --start-period=5s --retries=3 \
  CMD timeout 5 bash -c "</dev/tcp/localhost/9090" || exit 1

# Run the server
CMD ["sova-relay", "--host", "0.0.0.0", "--port", "9090", "--http-port", "8080"]