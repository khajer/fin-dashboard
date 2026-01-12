# Docker Documentation for fin-dashboard

This document explains how to build and run the fin-dashboard project using Docker.

## Overview

The fin-dashboard project uses Docker to containerize both the web server and worker processes. The setup includes:

- **Web Server**: An Actix-web based server serving on port 8080
- **Workers**: 5 worker instances that connect to the server via WebSocket

## Prerequisites

- Docker installed (version 20.10 or later recommended)
- Docker Compose installed (version 1.29 or later recommended)

## Quick Start

### Option 1: Using Docker Compose (Recommended)

The fastest way to get everything running is using docker-compose:

```bash
# Build and start all services
docker-compose up -d

# View logs
docker-compose logs -f

# Stop all services
docker-compose down

# Stop and remove volumes
docker-compose down -v
```

### Option 2: Building Images Manually

If you prefer to build images separately:

```bash
# Build web server image
docker build -t fin-dashboard:latest .

# Build worker image
docker build -t fin-dashboard-worker:latest ./workers

# Run web server
docker run -d -p 8080:8080 --name fin-dashboard-web fin-dashboard:latest

# Run 5 workers
for i in {1..5}; do
  docker run -d --name fin-dashboard-worker-$i fin-dashboard-worker:latest
done
```

### Option 3: Using the Build Script

We provide a convenient script to build both images:

```bash
# Build all Docker images
./docker-build.sh

# Then run with docker-compose
docker-compose up -d
```

## Docker Images

### Web Server Image (`fin-dashboard:latest`)

- **Base Image**: `debian:bookworm-slim`
- **Port**: 8080
- **Binary**: `/app/fin-dashboard`
- **Health Check**: Checks `/health` endpoint every 30 seconds

### Worker Image (`fin-dashboard-worker:latest`)

- **Base Image**: `debian:bookworm-slim`
- **Binary**: `/app/b0t`
- **Replicas**: 5 (managed by docker-compose)
- **Dependency**: Waits for web server to be healthy

## Docker Compose Services

### Web Service

```yaml
web:
  - Port: 8080 (host) -> 8080 (container)
  - Restart Policy: unless-stopped
  - Health Check: HTTP GET /health
  - Network: fin-dashboard-network
```

### Workers Service

```yaml
workers:
  - Replicas: 5
  - Restart Policy: on-failure (max 3 attempts)
  - Dependency: web (waits for healthy status)
  - Network: fin-dashboard-network
```

## Docker Compose Commands

### Building and Starting

```bash
# Build images and start services
docker-compose up -d --build

# Start services without building
docker-compose up -d

# Start specific service
docker-compose up -d web
docker-compose up -d workers
```

### Viewing Logs

```bash
# Follow all logs
docker-compose logs -f

# Follow specific service logs
docker-compose logs -f web
docker-compose logs -f workers

# View last 100 lines
docker-compose logs --tail=100
```

### Stopping and Removing

```bash
# Stop services (keeps containers)
docker-compose stop

# Stop and remove containers
docker-compose down

# Stop, remove containers, networks, and volumes
docker-compose down -v

# Remove orphaned containers
docker-compose down --remove-orphans
```

### Managing Services

```bash
# Restart all services
docker-compose restart

# Restart specific service
docker-compose restart web

# Scale workers (adjust replica count)
docker-compose up -d --scale workers=10

# View running containers
docker-compose ps

# Execute command in container
docker-compose exec web /bin/bash
docker-compose exec workers /bin/bash
```

## Architecture

```
┌─────────────────────────────────────────┐
│         Docker Network                  │
│     fin-dashboard-network              │
│                                         │
│  ┌─────────────┐     ┌─────────────┐    │
│  │   Web       │◄────│  Worker 1   │    │
│  │  Server     │     └─────────────┘    │
│  │  :8080      │     ┌─────────────┐    │
│  └─────────────┘◄────│  Worker 2   │    │
│       ▲           └─────────────┘    │
│       │           ┌─────────────┐    │
│       │    ◄──────│  Worker 3   │    │
│       │           └─────────────┘    │
│       │           ┌─────────────┐    │
│       │    ◄──────│  Worker 4   │    │
│       │           └─────────────┘    │
│       │           ┌─────────────┐    │
│       └────◄──────│  Worker 5   │    │
│                   └─────────────┘    │
└─────────────────────────────────────────┘
```

## Troubleshooting

### Workers cannot connect to web server

**Problem**: Workers fail to connect with connection errors.

**Solutions**:
1. Check if web server is healthy:
   ```bash
   docker-compose ps web
   docker-compose logs web
   ```

2. Verify network connectivity:
   ```bash
   docker-compose exec workers ping web
   ```

3. Check web server logs:
   ```bash
   docker-compose logs -f web
   ```

### Images not building

**Problem**: Build fails with compilation errors.

**Solutions**:
1. Ensure Docker has enough resources (memory, disk space)
2. Try building without cache:
   ```bash
   docker-compose build --no-cache
   ```
3. Check Docker daemon logs:
   ```bash
   docker system events
   ```

### Containers not starting

**Problem**: Services fail to start or immediately exit.

**Solutions**:
1. Check container logs:
   ```bash
   docker-compose logs <service-name>
   ```

2. Inspect container status:
   ```bash
   docker-compose ps
   ```

3. Run container interactively for debugging:
   ```bash
   docker-compose run --rm web /bin/bash
   ```

### Port conflicts

**Problem**: Port 8080 already in use.

**Solutions**:
1. Stop conflicting services:
   ```bash
   # Check what's using port 8080
   lsof -i :8080
   ```

2. Change port mapping in docker-compose.yml:
   ```yaml
   ports:
     - "8081:8080"  # Use 8081 instead
   ```

## Performance Tuning

### Memory Limits

Add resource limits to docker-compose.yml:

```yaml
services:
  web:
    deploy:
      resources:
        limits:
          memory: 512M
        reservations:
          memory: 256M

  workers:
    deploy:
      resources:
        limits:
          memory: 256M
        reservations:
          memory: 128M
```

### Worker Scaling

Scale workers based on load:

```bash
# Scale to 10 workers
docker-compose up -d --scale workers=10

# Scale down to 3 workers
docker-compose up -d --scale workers=3
```

### Log Rotation

Prevent disk space issues with log rotation:

```yaml
services:
  web:
    logging:
      driver: "json-file"
      options:
        max-size: "10m"
        max-file: "3"

  workers:
    logging:
      driver: "json-file"
      options:
        max-size: "10m"
        max-file: "3"
```

## Maintenance

### Cleaning Up

```bash
# Remove unused images
docker image prune -a

# Remove stopped containers
docker container prune

# Remove unused volumes
docker volume prune

# Remove all unused resources
docker system prune -a --volumes
```

### Updating Images

```bash
# Rebuild images with latest code
docker-compose build --no-cache

# Restart services with new images
docker-compose up -d

# Monitor startup
docker-compose logs -f
```

## Production Considerations

1. **Security**:
   - Run containers as non-root users
   - Use secrets for sensitive data
   - Keep images updated with security patches

2. **Monitoring**:
   - Set up log aggregation (ELK, Loki, etc.)
   - Monitor container health and resource usage
   - Set up alerts for failures

3. **Networking**:
   - Use separate networks for different environments
   - Consider using a reverse proxy (Nginx, Traefik)
   - Implement proper firewall rules

4. **Persistence**:
   - Use named volumes for persistent data if needed
   - Backup configuration and data regularly

## Additional Resources

- [Docker Documentation](https://docs.docker.com/)
- [Docker Compose Documentation](https://docs.docker.com/compose/)
- [Docker Best Practices](https://docs.docker.com/develop/dev-best-practices/)
- [Rust Docker Guidelines](https://doc.rust-lang.org/cargo/guide/building-docker-image.html)

## Support

For issues related to:
- **Docker setup**: Check this documentation first
- **Application bugs**: Review application logs and source code
- **Infrastructure**: Contact DevOps team or check infrastructure docs

---

Last updated: $(date)