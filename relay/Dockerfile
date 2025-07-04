# Multi-stage build for BuboCore Relay
FROM rust:1.75-slim as builder

# Install system dependencies
RUN apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev \
    && rm -rf /var/lib/apt/lists/*

# Set working directory
WORKDIR /app

# Copy source code
COPY . .

# Build the relay server
RUN cargo build --release -p bubocore-relay

# Runtime stage
FROM debian:bookworm-slim

# Install runtime dependencies
RUN apt-get update && apt-get install -y \
    ca-certificates \
    && rm -rf /var/lib/apt/lists/*

# Create non-root user
RUN useradd -r -s /bin/false relay

# Copy binary from builder
COPY --from=builder /app/target/release/bubocore-relay /usr/local/bin/

# Set ownership
RUN chown relay:relay /usr/local/bin/bubocore-relay

# Switch to non-root user
USER relay

# Expose port
EXPOSE 9090

# Health check
HEALTHCHECK --interval=30s --timeout=10s --start-period=5s --retries=3 \
  CMD timeout 5 bash -c "</dev/tcp/localhost/9090" || exit 1

# Run the server
CMD ["bubocore-relay", "--host", "0.0.0.0", "--port", "9090"]