# Sova Relay Server

A relay server for enabling remote collaboration between Sova instances.

## Overview

The relay server acts as a message broker between multiple Sova instances running in different locations. It enables real-time synchronization of sequencer state while maintaining local audio playback and timing independence.

## Features

- **Version Enforcement**: Strict version matching prevents compatibility issues
- **Rate Limiting**: Protects against message flooding
- **Connection Management**: Automatic cleanup of stale connections
- **Minimal Latency**: Efficient message routing with low overhead
- **Graceful Shutdown**: Handles Ctrl+C and connection cleanup

## Building

From the Sova root directory:

```bash
cargo build --release -p sova-relay
```

## Usage

### Basic Usage

```bash
# Start relay server on default port
./target/release/sova-relay

# Custom host and port
./target/release/sova-relay --host 0.0.0.0 --port 9090

# With custom limits
./target/release/sova-relay \
    --max-instances 50 \
    --rate-limit 2000 \
    --log-level debug
```

### Command Line Options

- `--host`: IP address to bind to (default: 0.0.0.0)
- `--port`: Port to listen on (default: 9090)
- `--max-instances`: Maximum concurrent instances (default: 20)
- `--rate-limit`: Messages per minute per instance (default: 1000)
- `--log-level`: Log level (default: info)

## Protocol

### Message Format

Messages use MessagePack serialization with a 4-byte big-endian length prefix:

```
[4 bytes: message length][N bytes: MessagePack data]
```

### Message Types

1. **RegisterInstance**: Initial handshake from Sova instance
2. **RegistrationResponse**: Server response with instance ID
3. **StateUpdate**: State change to be relayed to other instances
4. **StateBroadcast**: Broadcasted state change from another instance
5. **Ping/Pong**: Connection health checks
6. **Error**: Error messages
7. **InstanceDisconnected**: Notification of instance disconnection

### Connection Flow

1. Sova instance connects to relay
2. Sends `RegisterInstance` with name and version
3. Relay validates version and name uniqueness
4. If successful, sends `RegistrationResponse` with instance ID
5. Instance can now send `StateUpdate` messages
6. Relay broadcasts updates to all other instances

## Architecture

```
┌─────────────────┐     ┌─────────────────┐     ┌─────────────────┐
│ Sova NYC        │────▶│     Relay       │◀────│ Sova Tokyo      │
│ Instance        │     │     Server      │     │ Instance        │
└─────────────────┘     └─────────────────┘     └─────────────────┘
                               │
                               ▼
                        ┌─────────────────┐
                        │ Sova     London │
                        │ Instance        │
                        └─────────────────┘
```

## Configuration

### Environment Variables

- `RUST_LOG`: Controls logging level (e.g., `debug`, `info`, `warn`, `error`)

### Rate Limiting

Each instance is limited to:
- 1000 messages per minute (configurable)
- 1MB maximum message size
- Rate limit windows reset every 60 seconds

### Version Checking

The relay server enforces strict version matching:
- All connecting instances must have identical version strings
- Version is extracted from `CARGO_PKG_VERSION` at compile time
- Mismatched versions are immediately rejected

## Error Handling

### Common Errors

- **Version Mismatch**: Instance version doesn't match relay version
- **Name Taken**: Instance name already in use
- **Max Instances**: Server at capacity
- **Rate Limit**: Instance sending too many messages
- **Message Too Large**: Message exceeds size limit

### Logging

The relay server provides structured logging:

```
2024-01-15T10:30:00Z INFO  sova_relay: Starting Sova Relay Server v0.1.0
2024-01-15T10:30:00Z INFO  sova_relay: Listening on 0.0.0.0:9090
2024-01-15T10:30:15Z INFO  sova_relay: New connection from 192.168.1.100:54321
2024-01-15T10:30:15Z INFO  sova_relay: Instance 'alice-studio' registered with ID 123e4567-e89b-12d3-a456-426614174000
```

## Security Considerations

### Current Implementation

- Basic rate limiting per instance
- Message size validation
- Connection cleanup on disconnect

### Future Enhancements

- TLS encryption for connections
- Token-based authentication
- IP-based access control
- Enhanced audit logging

## Development

### Running Tests

```bash
cargo test -p sova-relay
```

### Development Mode

```bash
# Run with debug logging
RUST_LOG=debug cargo run -p sova-relay
```

### Adding Features

The crate is structured for easy extension:

- `types.rs`: Message types and data structures
- `relay.rs`: Core server logic
- `main.rs`: CLI and application entry point

## Performance

### Resource Usage

- **Memory**: ~50-100MB for typical usage
- **CPU**: Low, mostly I/O bound
- **Network**: Minimal overhead per message

### Scaling Limits

- Tested up to 20 concurrent instances
- Theoretical limit depends on network bandwidth
- Single-threaded design sufficient for target load

## Deployment

### Docker Deployment

#### Option 1: Pre-built Binary (Recommended for VPS)

To avoid memory issues when compiling Rust on a VPS:

1. **Build locally or on the host machine:**
   ```bash
   cd /path/to/Sova
   CARGO_BUILD_JOBS=1 cargo build --release -p sova-relay
   ```

2. **Use the lightweight Dockerfile:**
   ```bash
   # Build image with pre-compiled binary
   docker build -f relay/Dockerfile.prebuilt -t sova-relay .
   ```

3. **Start the container:**
   ```bash
   docker run -d \
     --name sova-relay \
     -p 9090:9090 \
     -e RUST_LOG=info \
     sova-relay
   ```

#### Option 2: Full Compilation (Requires >4GB RAM)

If you have sufficient memory:

```bash
# Build image with full compilation
docker build -f relay/Dockerfile -t sova-relay .
```

#### Integration with docker-compose

To integrate into an existing docker-compose stack:

```yaml
services:
  sova-relay:
    build:
      context: ./Sova
      dockerfile: relay/Dockerfile.prebuilt  # Uses pre-compiled binary
    container_name: sova-relay
    restart: unless-stopped
    environment:
      - RUST_LOG=info
    networks:
      - proxy  # If using Traefik
    deploy:
      resources:
        limits:
          memory: 256M
          cpus: '0.5'
    healthcheck:
      test: ["CMD", "timeout", "5", "bash", "-c", "</dev/tcp/localhost/9090"]
      interval: 30s
      timeout: 10s
      retries: 3
    # If using Traefik for reverse proxy:
    labels:
      - traefik.enable=true
      - traefik.http.routers.sova-relay.rule=Host(`relay.your-domain.fr`)
      - traefik.http.routers.sova-relay.entrypoints=websecure
      - traefik.http.routers.sova-relay.tls.certresolver=myresolver
      - traefik.http.services.sova-relay.loadbalancer.server.port=9090
```

#### Memory Issues Troubleshooting

**Problem:** Rust compilation can consume 1-2GB RAM per parallel process and crash VPS with limited memory.

**Solutions:**

1. **Single-threaded compilation:**
   ```bash
   CARGO_BUILD_JOBS=1 cargo build --release -p sova-relay
   ```

2. **Add temporary swap (temporary):**
   ```bash
   sudo fallocate -l 2G /swapfile
   sudo chmod 600 /swapfile
   sudo mkswap /swapfile
   sudo swapon /swapfile
   # Compile, then disable
   sudo swapoff /swapfile
   sudo rm /swapfile
   ```

3. **Use Dockerfile.prebuilt:** (Recommended)
   - Compile on your local machine
   - Copy only the binary into the final image
   - Final image: ~100MB vs 2GB+ for full compilation

#### Access and Testing

Once deployed, the relay will be accessible:

- **Direct port:** `http://your-server:9090`
- **Via Traefik:** `https://relay.your-domain.fr`
- **Health check:** Service exposes a healthcheck on port 9090

#### Logs and Monitoring

```bash
# View service logs
docker logs sova-relay

# Real-time logs
docker logs -f sova-relay

# Resource statistics
docker stats sova-relay
```

### Production Configuration

```bash
# Recommended environment variables
RUST_LOG=info                    # Appropriate log level
SOVA_MAX_INSTANCES=50       # Instance limit
SOVA_RATE_LIMIT=2000        # Messages per minute
```

## License

Same as Sova main project.