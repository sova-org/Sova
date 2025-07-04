# BuboCore Relay Server

A relay server for enabling remote collaboration between BuboCore instances.

## Overview

The relay server acts as a message broker between multiple BuboCore instances running in different locations. It enables real-time synchronization of sequencer state while maintaining local audio playback and timing independence.

## Features

- **Version Enforcement**: Strict version matching prevents compatibility issues
- **Rate Limiting**: Protects against message flooding
- **Connection Management**: Automatic cleanup of stale connections
- **Minimal Latency**: Efficient message routing with low overhead
- **Graceful Shutdown**: Handles Ctrl+C and connection cleanup

## Building

From the BuboCore root directory:

```bash
cargo build --release -p bubocore-relay
```

## Usage

### Basic Usage

```bash
# Start relay server on default port
./target/release/bubocore-relay

# Custom host and port
./target/release/bubocore-relay --host 0.0.0.0 --port 9090

# With custom limits
./target/release/bubocore-relay \
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

1. **RegisterInstance**: Initial handshake from BuboCore instance
2. **RegistrationResponse**: Server response with instance ID
3. **StateUpdate**: State change to be relayed to other instances
4. **StateBroadcast**: Broadcasted state change from another instance
5. **Ping/Pong**: Connection health checks
6. **Error**: Error messages
7. **InstanceDisconnected**: Notification of instance disconnection

### Connection Flow

1. BuboCore instance connects to relay
2. Sends `RegisterInstance` with name and version
3. Relay validates version and name uniqueness
4. If successful, sends `RegistrationResponse` with instance ID
5. Instance can now send `StateUpdate` messages
6. Relay broadcasts updates to all other instances

## Architecture

```
┌─────────────────┐     ┌─────────────────┐     ┌─────────────────┐
│ BuboCore NYC    │────▶│     Relay       │◀────│ BuboCore Tokyo  │
│ Instance        │     │     Server      │     │ Instance        │
└─────────────────┘     └─────────────────┘     └─────────────────┘
                               │
                               ▼
                        ┌─────────────────┐
                        │ BuboCore London │
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
2024-01-15T10:30:00Z INFO  bubocore_relay: Starting BuboCore Relay Server v0.1.0
2024-01-15T10:30:00Z INFO  bubocore_relay: Listening on 0.0.0.0:9090
2024-01-15T10:30:15Z INFO  bubocore_relay: New connection from 192.168.1.100:54321
2024-01-15T10:30:15Z INFO  bubocore_relay: Instance 'alice-studio' registered with ID 123e4567-e89b-12d3-a456-426614174000
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
cargo test -p bubocore-relay
```

### Development Mode

```bash
# Run with debug logging
RUST_LOG=debug cargo run -p bubocore-relay
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

See [deployment.md](../deployment.md) for Docker and VPS deployment instructions.

## License

Same as BuboCore main project.