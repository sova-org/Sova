# Quick Deployment Guide

## Before You Start

1. **Check your VPS memory:**
   ```bash
   free -h
   ```
   - If you have < 4GB RAM, use the prebuilt option
   - If you have ≥ 4GB RAM, you can use either option

## Deployment Steps

### Step 1: Build the Binary (Prebuilt Option)

```bash
# Navigate to Sova root
cd /path/to/Sova

# Build with single thread to avoid memory issues
CARGO_BUILD_JOBS=1 cargo build --release -p sova-relay
```

### Step 2: Choose Your Deployment Method

#### Option A: Standalone Docker

```bash
# Build and run standalone
cd relay
docker-compose up -d sova-relay-prebuilt
```

#### Option B: Integration with Existing Stack

Add to your main `docker-compose.yml`:

```yaml
sova-relay:
  build:
    context: ./sova
    dockerfile: relay/Dockerfile.prebuilt
  container_name: sova-relay
  restart: unless-stopped
  environment:
    - RUST_LOG=info
  networks:
    - proxy
  # Add Traefik labels if needed
```

### Step 3: Verify Deployment

```bash
# Check container status
docker ps | grep sova-relay

# Check logs
docker logs sova-relay

# Test connectivity
curl http://localhost:9090
# or with your domain
curl https://relay.your-domain.com
```

## Troubleshooting

### Memory Issues During Build
- Use `CARGO_BUILD_JOBS=1`
- Add temporary swap if needed
- Use the prebuilt Dockerfile

### Container Won't Start
- Check logs: `docker logs sova-relay`
- Verify binary exists: `ls -la target/release/sova-relay`
- Check port conflicts: `netstat -tulpn | grep 9090`

### Network Issues
- Verify Traefik configuration
- Check firewall rules
- Ensure DNS points to your server

## Performance Monitoring

```bash
# Resource usage
docker stats sova-relay

# Connection monitoring
docker logs -f sova-relay | grep "connection"

# Health check
docker inspect sova-relay | grep -A 5 Health
```