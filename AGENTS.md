# AGENT.md

This document provides comprehensive guidance for AI agents working on the Fin-Dashboard codebase. It complements `CLAUDE.md` by providing deeper technical context, operational procedures, and decision-making frameworks.

## Table of Contents

- [Project Overview](#project-overview)
- [System Architecture](#system-architecture)
- [Technology Stack](#technology-stack)
- [File Structure](#file-structure)
- [Development Workflow](#development-workflow)
- [Common Tasks](#common-tasks)
- [Testing Guidelines](#testing-guidelines)
- [Debugging & Troubleshooting](#debugging--troubleshooting)
- [Performance Considerations](#performance-considerations)
- [Deployment & Operations](#deployment--operations)
- [Security Considerations](#security-considerations)
- [Known Issues & Limitations](#known-issues--limitations)
- [Future Improvements](#future-improvements)

---

## Project Overview

Fin-Dashboard is a **real-time cryptocurrency price monitoring system** built with Rust. It demonstrates a distributed architecture with:

- **Centralized WebSocket server** (Actix-web based) managing connections and broadcasts
- **Distributed worker bots** that fetch price data from Binance API
- **Web dashboard** for real-time price visualization

### Core Design Philosophy

1. **Separation of Concerns**: Workers fetch data, server manages state, dashboards visualize
2. **Real-time Communication**: WebSocket-based for low-latency updates
3. **Scalability**: Horizontal scaling of workers via Docker/Podman
4. **State Management**: Shared state using `Arc<Mutex<>>` for thread safety

### System Constraints

- Server runs on port 8080
- Supports 8 stock symbols by default (configurable in `main.rs`)
- Workers fetch prices every 1 second
- Dashboard clients receive broadcasts for all symbols

---

## System Architecture

### High-Level Flow

```
┌──────────────┐     ┌──────────────┐     ┌──────────────────────┐
│   Workers    │────▶│    Server    │────▶│    Dashboard UI      │
│   (Bots)     │ WS  │  (Actix-web) │ WS  │   (Web Browser)      │
└──────────────┘     └──────┬───────┘     └──────────────────────┘
                           │
                           ▼
                    ┌──────────────┐
                    │  Binance API │
                    │  (External)  │
                    └──────────────┘
```

### Component Interactions

#### 1. Server Initialization
- Loads stock symbols into `stocklist: Arc<Mutex<Vec<&'static str>>>`
- Initializes empty `dashboard_clients: Arc<Mutex<Vec<Session>>>`
- Starts HTTP server with WebSocket endpoint `/ws`
- Serves static dashboard HTML and CSS files

#### 2. Worker Connection Flow
```
Worker → WebSocket Connect → Send Login {"username": "bot"}
       → Receive Assignment {"status": "success", "cmd": "BTCUSDT"}
       → Start Fetching Prices → Send Updates Every 1s
       → On Disconnect → Reconnect After 1s
```

#### 3. Dashboard Connection Flow
```
Dashboard → WebSocket Connect → Send Login {"username": "dashboard"}
          → Receive Confirmation → Added to Broadcast List
          → Receive All Price Updates → Display Real-time
          → Auto-reconnect after 3s on disconnect
```

#### 4. Broadcast Mechanism
```
Worker Sends Price → Server Receives → Loop Through dashboard_clients
                   → Send to Each Dashboard → Update UI
```

### State Management

**Thread Safety**: All shared state wrapped in `Arc<Mutex<>>`

```rust
// Symbol pool (removed when assigned to worker)
Arc<Mutex<Vec<&'static str>>>

// Active dashboard connections
Arc<Mutex<Vec<Session>>>
```

### Container Networking

**Docker/Podman Network**:
- Network name: `fin-dashboard-network`
- Type: Bridge network
- Services: `web` and `workers` communicate via service names
- Health checks ensure workers only connect when server is ready

---

## Technology Stack

### Server Dependencies

| Crate | Version | Purpose |
|-------|---------|---------|
| `actix-web` | 4.x | HTTP server framework |
| `actix-ws` | 0.3.0 | WebSocket support |
| `tokio` | 1.0 | Async runtime |
| `serde` | 1.0 | Serialization/deserialization |
| `serde_json` | 1.0 | JSON handling |
| `tracing` | 0.1 | Structured logging |
| `tracing-subscriber` | 0.3 | Log formatting |
| `futures-util` | 0.3 | Async utilities |

### Worker Dependencies

| Crate | Version | Purpose |
|-------|---------|---------|
| `tokio-tungstenite` | 0.21 | WebSocket client |
| `tokio` | 1.0 | Async runtime (full features) |
| `reqwest` | 0.11 | HTTP client for Binance API |
| `serde` | 1.0 | Serialization |
| `serde_json` | 1.0 | JSON handling |
| `url` | 2.4 | URL parsing |
| `clap` | 4.x | Command-line argument parsing |
| `futures-util` | 0.3 | Async utilities |

### Toolchain

- **Rust Version**: 1.82
- **Rust Edition**: 2024
- **Docker Base Image**: `rust:1.82-slim` (builder), `debian:bookworm-slim` (runtime)

### External Services

- **Binance API**: `https://api.binance.com/api/v3/ticker/price`
- Returns JSON: `{"symbol": "BTCUSDT", "price": "42000.00"}`

---

## File Structure

```
fin-dashboard/
├── src/
│   ├── main.rs              # Server entry point, HTTP routes, state init
│   └── ws_handler.rs        # WebSocket logic, authentication, message routing
├── workers/
│   ├── src/
│   │   └── main.rs          # Worker bot implementation
│   ├── Cargo.toml           # Worker dependencies (package name: b0t)
│   └── Dockerfile           # Worker container build
├── static/
│   ├── dashboard.html       # Dashboard UI (embedded via include_str!)
│   └── dashboard.css        # Dashboard styling (served as static file)
├── Cargo.toml               # Server dependencies
├── Dockerfile               # Server container build
├── docker-compose.yml       # Docker Compose orchestration
├── podman-compose.yml       # Podman Compose orchestration (alternative)
├── run_all.sh               # Start server + workers locally
├── docker-build.sh          # Build Docker images
├── CLAUDE.md                # Developer guidance
├── DOCKER.md                # Docker documentation
├── AGENTS.md                # This file - agent guidance
└── README.md                # Project overview
```

### File Responsibilities

#### `src/main.rs`
- HTTP server configuration
- Route definitions: `/`, `/health`, `/dashboard`, `/ws`
- Shared state initialization
- Application data setup with `web::Data::new()`
- Logging setup with `tracing`
- **Important**: HOST is hardcoded to `127.0.0.1:8080` for local development

#### `src/ws_handler.rs`
- WebSocket connection handling
- User authentication (bot vs dashboard)
- Symbol assignment to workers
- Price message parsing and broadcasting
- Session management
- **Pattern**: Uses `rt::spawn` for async task spawning
- **Pattern**: Uses `.unwrap()` extensively (production code should handle errors)

#### `workers/src/main.rs`
- WebSocket client implementation
- Binance API integration
- Price fetching interval logic (1s via `Duration::from_millis(1000)`)
- Auto-reconnection on failure
- CLI argument support (via clap, though minimally used)
- **Binary name**: `b0t` (not "worker")
- **Pattern**: Uses `tokio::spawn` for async task spawning (inconsistent with server)

#### `static/dashboard.html`
- WebSocket client implementation (JavaScript)
- Real-time price display
- Connection status indicators
- **Hardcoded URL**: `ws://localhost:8080/ws` (may need updating for production)
- **Auto-reconnect**: 3-second delay on disconnect

#### `static/dashboard.css`
- Dashboard styling (served separately from HTML)
- Gradient background, card-based layout
- Responsive grid design

#### `Dockerfile` (Server)
- Multi-stage build for optimization
- **Stage 1**: `rust:1.82-slim` - builds binary with dependency caching
- **Stage 2**: `debian:bookworm-slim` - minimal runtime with ca-certificates
- Serves binary on port 8080

#### `workers/Dockerfile`
- Multi-stage build matching server pattern
- **Binary output**: `/app/b0t`
- Same runtime as server

#### `docker-compose.yml`
- Orchestrates web server + 5 workers
- **Network**: `fin-dashboard-network` (bridge)
- **Health check**: Server must be healthy before workers start
- **Workers**: Configured with `deploy.replicas: 5`
- **Restart policy**: `unless-stopped`

#### `podman-compose.yml`
- Alternative to docker-compose.yml for Podman users
- **Key difference**: Does NOT use `deploy.replicas` (Podman Compose limitation)
- **Scaling**: Use `podman-compose up -d --scale workers=5` instead

---

## Development Workflow

### Prerequisites

1. **Required Tools**:
   - Rust toolchain 1.82 or later
   - Cargo (comes with Rust)
   - Optional: Docker (20.10+) or Podman
   - Optional: Docker Compose or Podman Compose
   - `websocat` for WebSocket testing (`brew install websocat` or `cargo install websocat`)

2. **Setup**:
   ```bash
   # Clone repository
   git clone <repository-url>
   cd fin-dashboard

   # Verify Rust version
   rustc --version  # Should be 1.82+
   ```

### Local Development

#### Option 1: Run Server and Workers Separately

```bash
# Terminal 1: Start server
cargo run

# Terminal 2: Start worker
cd workers
cargo run

# Repeat for additional workers in separate terminals
```

#### Option 2: Using Convenience Script

```bash
# Build release binaries and start server + 5 workers
./run_all.sh

# Press Ctrl+C to stop all processes
```

#### Option 3: Docker/Podman

```bash
# Using Docker
docker-compose up -d

# Using Podman
podman-compose up -d --scale workers=5

# View logs
docker-compose logs -f
# or
podman-compose logs -f

# Stop services
docker-compose down
# or
podman-compose down
```

### Accessing the Application

1. **Dashboard UI**: Open browser to `http://localhost:8080/dashboard`
2. **WebSocket Endpoint**: `ws://127.0.0.1:8080/ws`
3. **Health Check**: `http://localhost:8080/health`

### Code Modification Workflow

**When modifying the codebase, follow this order:**

1. Identify the affected component (server/worker/frontend)
2. Make changes to source code
3. Run relevant tests
4. Build and test locally
5. If using containers, rebuild images and restart services
6. Verify functionality

#### Modifying Server Code

```bash
# Edit src/main.rs or src/ws_handler.rs
cargo run  # Manual restart required (no auto-reload)
```

#### Modifying Worker Code

```bash
# Edit workers/src/main.rs
cd workers
cargo run
```

#### Modifying Frontend

```bash
# Edit static/dashboard.html or static/dashboard.css
# For local development: just refresh browser after server restart
# For containers: rebuild server image
docker-compose up -d --build web
```

### Testing

```bash
# Run all tests
cargo test

# Run tests for specific module
cargo test ws_handler

# Run tests with output
cargo test -- --nocapture

# Run specific test
cargo test test_parse_login_text
```

### Building

```bash
# Debug build (faster compilation)
cargo build

# Release build (optimized)
cargo build --release

# Build workers
cd workers
cargo build --release
```

---

## Common Tasks

### Task 1: Add a New Cryptocurrency Symbol

**Location**: `src/main.rs` (lines 58-67)

```rust
let stocklist = Arc::new(Mutex::new(vec![
    "BTCUSDT",
    "ETHUSDT",
    "BNBUSDT",
    "SOLUSDT",
    "XRPUSDT",
    "DOTUSDT",
    "ADAUSDT",
    "TRXUSDT",
    // Add new symbol here: "DOGEUSDT",
]));
```

**Impact**: Increases symbol pool, allows more workers to connect (one symbol per worker)

**Note**: Must verify symbol exists on Binance API

### Task 2: Change Price Update Interval

**Location**: `workers/src/main.rs` (line 150)

```rust
// Current: 1 second (1000 milliseconds)
sleep(Duration::from_millis(1000)).await;

// Change to different interval:
sleep(Duration::from_secs(5)).await;  // 5 seconds
```

**Impact**: Affects API call frequency and update rate

### Task 3: Add a New HTTP Endpoint

**Location**: `src/main.rs`

```rust
// Add handler function
#[get("/metrics")]
async fn metrics() -> impl Responder {
    HttpResponse::Ok().body("Metrics endpoint")
}

// Register in App::new()
.service(metrics)
```

### Task 4: Change Server Port

**Location**: `src/main.rs` (line 10)

```rust
// Development: Change to desired port
const HOST: &str = "127.0.0.1:8080";

// Container deployment: Use 0.0.0.0 to listen on all interfaces
const HOST: &str = "0.0.0.0:8080";
```

**Also update**: `docker-compose.yml` port mapping, `podman-compose.yml`, `DOCKER.md` documentation

### Task 5: Add Authentication

**Location**: `src/ws_handler.rs` (parse_login_text function)

```rust
// Implement token-based or credential-based auth
fn parse_login_text(text: &str) -> Result<LoginData, AuthError> {
    // Add validation logic here
    if !validate_token(token) {
        return Err(AuthError::InvalidToken);
    }
    // ...
}
```

### Task 6: Implement Connection Cleanup

**Location**: `src/ws_handler.rs`

Add cleanup logic when dashboard disconnects or periodically:
```rust
// Remove closed sessions from dashboard_clients
dashboard_clients.retain(|session| !session.is_closed());
```

### Task 7: Add Unit Tests

**Location**: `src/ws_handler.rs` (lines 138-249)

```rust
#[cfg(test)]
mod tests {
    #[test]
    fn test_new_functionality() {
        // Test implementation
    }
}
```

### Task 8: Customize Dashboard UI

**Location**: `static/dashboard.html` and `static/dashboard.css`

Modify HTML/JavaScript/CSS to:
- Change layout/styling
- Add charts/graphs
- Implement historical data display
- Add alerts/notifications
- Update WebSocket URL for production

### Task 9: Change API Endpoint

**Location**: `workers/src/main.rs` (fetch_price function, line 155)

```rust
async fn fetch_price(symbol: String) -> Result<BinancePriceResponse, reqwest::Error> {
    let client = Client::new();
    let url = format!(
        "https://api.binance.com/api/v3/ticker/price?symbol={symbol}"
    );
    // Change to different exchange or custom API here
    let response = client
        .get(url)
        .send()
        .await?
        .json::<BinancePriceResponse>()
        .await?;
    Ok(response)
}
```

### Task 10: Implement Graceful Shutdown

**Location**: `src/main.rs`

Add signal handling:
```rust
use tokio::signal;

// In main()
async fn main() -> std::io::Result<()> {
    // ... setup code ...

    // Handle Ctrl+C
    let handle = server.handle();
    tokio::spawn(async move {
        signal::ctrl_c().await.unwrap();
        info!("Shutting down gracefully...");
        handle.stop(true).await;
    });

    server.await
}
```

---

## Testing Guidelines

### Existing Test Coverage

**Location**: `src/ws_handler.rs` (lines 138-249)

Current tests cover:
- JSON parsing for login messages
- Command parsing for price updates
- Message format validation
- Stocklist manipulation
- Session management

### Running Tests

```bash
# All tests
cargo test

# Specific module
cargo test ws_handler

# With output
cargo test -- --nocapture

# Specific test
cargo test test_parse_login_text

# Run tests with backtrace on failure
cargo test -- --nocapture -- -Z unstable-options --report-time
```

### Writing New Tests

**Principles**:
1. Test public APIs and critical business logic
2. Mock external dependencies (Binance API)
3. Test edge cases (empty inputs, invalid formats)
4. Test concurrent access patterns for shared state
5. Keep tests fast and independent

**Example Test Pattern**:

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_symbol_assignment() {
        let stocklist = Arc::new(Mutex::new(vec!["BTCUSDT", "ETHUSDT"]));

        // Assign symbol
        let symbol = assign_symbol(&stocklist).unwrap();

        assert_eq!(symbol, "BTCUSDT");

        // Verify removal from pool
        let remaining = stocklist.lock().unwrap();
        assert_eq!(remaining.len(), 1);
    }

    #[test]
    fn test_no_symbols_available() {
        let empty_list = Arc::new(Mutex::new(Vec::new()));

        let result = assign_symbol(&empty_list);
        assert!(result.is_err());
    }
}
```

### Integration Testing

**Current State**: No automated integration tests

**Manual Testing Procedure**:
1. Start server: `cargo run` or `docker-compose up -d`
2. Start multiple workers (5+ recommended)
3. Open dashboard in browser: `http://localhost:8080/dashboard`
4. Verify real-time updates appear
5. Test reconnection: kill workers, verify they reconnect after 1s
6. Test broadcast: open multiple dashboard tabs, verify all receive updates

**WebSocket Manual Testing**:
```bash
# Install websocat
brew install websocat  # macOS
# or
cargo install websocat

# Connect as bot
echo '{"username": "bot"}' | websocat ws://127.0.0.1:8080/ws

# Connect as dashboard
echo '{"username": "dashboard"}' | websocat ws://127.0.0.1:8080/ws
```

### Test Data

**Mock Binance API Response**:
```json
{
  "symbol": "BTCUSDT",
  "price": "42000.50"
}
```

**Test Login Messages**:
```json
// Bot login
{"username": "bot"}

// Dashboard login
{"username": "dashboard"}

// Invalid login
{"username": "invalid_user"}
```

---

## Debugging & Troubleshooting

### Common Issues & Solutions

#### 1. Workers Cannot Connect to Server

**Symptoms**:
- Worker logs show connection errors
- Dashboard shows no price updates

**Diagnosis**:
```bash
# Check if server is running
curl http://127.0.0.1:8080/health

# Check server logs
docker-compose logs -f web

# Check worker logs
docker-compose logs -f workers
```

**Solutions**:
- Ensure server is running on port 8080
- Verify firewall settings
- Check WebSocket URL in worker code
- Verify network connectivity in Docker/Podman environment
- **Common issue**: HOST in main.rs is `127.0.0.1` for local dev, must be `0.0.0.0` for containers

#### 2. Symbol Pool Exhausted

**Symptoms**:
- 9th worker connects but receives no symbol assignment
- Server logs show empty stocklist

**Diagnosis**:
```bash
# Count connected workers
docker-compose ps | grep workers
```

**Solutions**:
- Add more symbols to `stocklist` in `main.rs`
- Reduce number of workers
- Implement symbol recycling when workers disconnect

#### 3. Dashboard Clients Not Receiving Updates

**Symptoms**:
- Workers are connected and sending data
- Dashboard shows no updates

**Diagnosis**:
```bash
# Check dashboard client count in server logs
docker-compose logs -f web | grep dashboard

# Verify WebSocket connection in browser console
# Open DevTools → Network → WS tab
```

**Solutions**:
- Verify dashboard login message format
- Check if dashboard is in broadcast list
- Implement connection cleanup for stale sessions
- Check browser console for JavaScript errors

#### 4. Memory Leaks

**Symptoms**:
- Server memory usage grows over time
- Performance degrades

**Diagnosis**:
```bash
# Monitor memory usage
docker stats fin-dashboard_web

# Check connection count
docker-compose exec web ps aux
```

**Solutions**:
- Implement periodic cleanup of closed sessions
- Add monitoring for connection count
- Consider using weak references for sessions

#### 5. High CPU Usage

**Symptoms**:
- Server or workers consuming excessive CPU

**Diagnosis**:
```bash
# Monitor CPU usage
docker stats

# Profile with flamegraph
cargo install flamegraph
cargo flamegraph
```

**Solutions**:
- Reduce update frequency
- Optimize JSON parsing
- Implement caching for API responses

#### 6. Port Already in Use

**Symptoms**:
- `cargo run` or docker-compose fails with port binding error

**Diagnosis**:
```bash
# Check what's using port 8080
lsof -i :8080  # macOS/Linux
netstat -ano | findstr :8080  # Windows
```

**Solutions**:
- Kill the process using the port
- Change port in `src/main.rs` and restart
- Use different port mapping in docker-compose.yml

### Debugging Tools

**Logging**:
```rust
// Use tracing for structured logging
use tracing::{info, warn, error, debug};

info!("Worker connected, assigned symbol: {}", symbol);
warn!("Failed to fetch price: {}", error);
error!("Connection lost, attempting reconnect");
debug!("Current symbol pool: {:?}", stocklist);
```

**WebSocket Testing**:
```bash
# Install websocat
brew install websocat  # macOS
cargo install websocat  # Cross-platform

# Test connection
websocat ws://127.0.0.1:8080/ws

# Send test message
echo '{"username": "bot"}' | websocat ws://127.0.0.1:8080/ws
```

**HTTP Testing**:
```bash
# Health check
curl http://127.0.0.1:8080/health

# Dashboard endpoint
curl http://127.0.0.1:8080/dashboard

# With verbose output
curl -v http://127.0.0.1:8080/health
```

**Docker Debugging**:
```bash
# Exec into container
docker-compose exec web /bin/bash
docker-compose exec workers /bin/bash

# Inspect container
docker-compose ps
docker inspect fin-dashboard_web_1

# View container logs
docker logs fin-dashboard_web_1

# View resource usage
docker stats
```

### Performance Profiling

**Build with profiling**:
```bash
cargo build --release
```

**Use flamegraph**:
```bash
cargo install flamegraph
cargo flamegraph --bin fin-dashboard
```

**Monitor resources**:
```bash
# CPU & Memory
docker stats

# Network traffic
docker stats --no-stream

# Real-time logs
docker-compose logs -f
```

---

## Performance Considerations

### Bottlenecks

1. **WebSocket Broadcast**
   - Linear iteration through `dashboard_clients`
   - Impact: O(n) per message, where n = number of dashboard clients
   - Mitigation: Use channel-based broadcast, limit max clients

2. **API Rate Limiting**
   - Binance API has rate limits
   - Impact: Workers may get 429 errors
   - Mitigation: Implement backoff, distribute requests across time

3. **Shared State Lock Contention**
   - `Arc<Mutex<>>` can cause contention under high load
   - Impact: Blocking operations delay message processing
   - Mitigation: Use `RwLock` for read-heavy workloads, or sharded state

4. **Error Handling Overhead**
   - Extensive `.unwrap()` usage can cause panics
   - Impact: Service crashes instead of graceful degradation
   - Mitigation: Implement proper error handling with recovery

### Optimization Strategies

**Server-Side**:
```rust
// 1. Use Tokio channels for broadcasting
use tokio::sync::broadcast;

// 2. Batch messages if possible
// 3. Implement connection pooling
// 4. Use async DNS resolution
// 5. Replace .unwrap() with proper error handling
```

**Worker-Side**:
```rust
// 1. Implement exponential backoff for API calls
// 2. Cache responses if data doesn't change frequently
// 3. Use connection pooling for HTTP client
// 4. Add request timeout to reqwest client
```

### Capacity Planning

**Current Limits**:
- 8 symbols (expandable)
- 1-second update interval
- 5 workers (default)

**Scaling Considerations**:
- Horizontal scaling: Add more workers via `docker-compose up -d --scale workers=N`
- Vertical scaling: Increase update frequency
- Network bandwidth: ~100 bytes/message × 5 workers × 1/s = 500 B/s
- Memory: ~1KB per connection

**Recommended Configurations**:

| Scenario | Workers | Update Interval | Expected Load |
|----------|---------|-----------------|---------------|
| Development | 2-3 | 1s | Low |
| Staging | 5 | 1s | Medium |
| Production | 10-20 | 100ms | High |

---

## Deployment & Operations

### Environment Selection

**Development**:
- Use `cargo run` for quick iteration
- HOST: `127.0.0.1:8080`
- Single terminal per process

**Production/Containerized**:
- Use Docker or Podman
- HOST: `0.0.0.0:8080` (listen on all interfaces)
- Multi-container orchestration
- Health checks and auto-restart

### Docker Workflow

#### Building Images

```bash
# Build both images
./docker-build.sh

# Or manually
docker-compose build

# Build without cache
docker-compose build --no-cache
```

#### Starting Services

```bash
# Start all services
docker-compose up -d

# Start with scaling
docker-compose up -d --scale workers=10

# View logs
docker-compose logs -f
docker-compose logs -f web
docker-compose logs -f workers
```

#### Managing Services

```bash
# Stop services
docker-compose stop

# Restart services
docker-compose restart

# Stop and remove containers
docker-compose down

# Stop and remove containers, networks, volumes
docker-compose down -v
```

### Podman Workflow

Podman Compose has some differences from Docker Compose:

#### Key Differences

1. **No `deploy.replicas` support**: Use `--scale` flag instead
2. **Rootless containers**: Runs as non-root user by default
3. **Different networking**: Uses Podman's network stack

#### Commands

```bash
# Build images
podman-compose build

# Start services (scale workers manually)
podman-compose up -d --scale workers=5

# View logs
podman-compose logs -f

# Stop services
podman-compose down
```

#### Scaling in Podman

```bash
# Scale to 10 workers
podman-compose up -d --scale workers=10

# Scale down to 3 workers
podman-compose up -d --scale workers=3
```

### Production Checklist

- [ ] Change HOST to `0.0.0.0:8080` for container deployment
- [ ] Configure proper logging levels (INFO/WARN/ERROR)
- [ ] Implement authentication for dashboard
- [ ] Set up monitoring and alerting
- [ ] Configure SSL/TLS for WebSocket
- [ ] Implement rate limiting
- [ ] Set up log aggregation
- [ ] Configure health checks
- [ ] Set up graceful shutdown
- [ ] Backup critical configuration
- [ ] Document runbooks

### Environment Variables

**Server**:
```bash
RUST_LOG=info                    # Log level (debug, info, warn, error)
HOST=0.0.0.0                     # Listen address
PORT=8080                        # Listen port
```

**Workers**:
```bash
RUST_LOG=info                    # Log level
SERVER_URL=ws://server:8080/ws   # WebSocket URL (for containers)
```

### Docker Production Configuration

**Modify `docker-compose.yml`**:
```yaml
services:
  web:
    environment:
      - RUST_LOG=warn            # Reduce log verbosity
      - HOST=0.0.0.0             # Listen on all interfaces
    deploy:
      resources:
        limits:
          memory: 512M
          cpus: '0.5'
        reservations:
          memory: 256M
          cpus: '0.25'
    restart: always              # Always restart on failure
    logging:
      driver: "json-file"
      options:
        max-size: "10m"
        max-file: "3"
```

### Monitoring Metrics

**Key Metrics to Track**:
- Active connections (workers + dashboards)
- Message throughput (messages/second)
- API latency (time to fetch price)
- WebSocket connection latency
- Error rates (failed connections, API errors)
- Resource usage (CPU, memory, network)

**Monitoring Tools**:
- Prometheus for metrics collection
- Grafana for visualization
- Loki for log aggregation
- Alertmanager for alerting

### Health Checks

**Current**: HTTP GET `/health` returns version

**Enhanced Health Check**:
```rust
#[get("/health")]
async fn health_check(
    stocklist: web::Data<Arc<Mutex<Vec<&'static str>>>>,
    dashboard_clients: web::Data<Arc<Mutex<Vec<Session>>>>,
) -> impl Responder {
    let status = HealthStatus {
        status: "ok".to_string(),
        version: Some(CURR_VERSION.to_string()),
        symbols_available: stocklist.lock().unwrap().len(),
        active_dashboards: dashboard_clients.lock().unwrap().len(),
    };
    Ok(web::Json(status))
}
```

### Log Management

**Log Rotation** (Docker Compose):
```yaml
services:
  web:
    logging:
      driver: "json-file"
      options:
        max-size: "10m"
        max-file: "3"
```

**Log Format**:
```
INFO  [timestamp] worker_connected: assigned=BTCUSDT
WARN  [timestamp] api_error: message=rate_limit_exceeded
ERROR [timestamp] connection_lost: peer=127.0.0.1:54321
```

### Backup & Recovery

**Backup Configuration**:
- Stock symbol list
- Server configuration
- Docker Compose/Podman Compose files
- Static files (dashboard.html, dashboard.css)

**Recovery Procedure**:
1. Restore configuration files
2. Rebuild Docker images
3. Start services with `docker-compose up -d`
4. Verify health checks
5. Monitor logs for issues

---

## Security Considerations

### Current Security Posture

**Weaknesses**:
- No authentication for dashboard or workers
- No TLS/SSL encryption
- No rate limiting
- No input validation on price data
- Open WebSocket endpoint
- Extensive `.unwrap()` usage can expose DoS vectors
- Hardcoded WebSocket URL in dashboard.html

### Security Improvements

#### 1. Authentication

**Add JWT token authentication**:

```rust
// Server-side
use jsonwebtoken::{decode, encode, Validation, Header};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
struct Claims {
    sub: String,    // username
    exp: usize,      // expiry
    typ: String,     // "bot" or "dashboard"
}

// Verify token on connection
fn verify_token(token: &str) -> Result<Claims, AuthError> {
    let secret = std::env::var("JWT_SECRET").unwrap();
    decode::<Claims>(token, &secret, &Validation::default())
        .map_err(|_| AuthError::InvalidToken)
}
```

**Client-side (workers)**:
```rust
// Generate JWT token before connecting
let token = generate_jwt_token("bot", &["BTCUSDT"]);
let ws_url = format!("ws://server:8080/ws?token={}", token);
```

#### 2. TLS/SSL Encryption

**Use reverse proxy (Nginx)**:

```nginx
server {
    listen 443 ssl;
    server_name dashboard.example.com;

    ssl_certificate /path/to/cert.pem;
    ssl_certificate_key /path/to/key.pem;

    location /ws {
        proxy_pass http://localhost:8080/ws;
        proxy_http_version 1.1;
        proxy_set_header Upgrade $http_upgrade;
        proxy_set_header Connection "upgrade";
    }
}
```

#### 3. Rate Limiting

**Implement per-IP rate limiting**:

```rust
use actix_web::middleware::Condition;

// In App::new()
.wrap(Condition::new(
    cfg!(feature = "rate-limit"),
    actix_web::middleware::DefaultHeaders::new()
        .add(("X-Rate-Limit", "100/minute"))
))
```

#### 4. Input Validation

**Validate price data**:

```rust
#[derive(Debug, Deserialize)]
struct PriceUpdate {
    symbol: String,
    price: String,
}

impl PriceUpdate {
    fn validate(&self) -> Result<(), ValidationError> {
        if self.symbol.len() > 10 {
            return Err(ValidationError::SymbolTooLong);
        }
        if self.price.parse::<f64>().is_err() {
            return Err(ValidationError::InvalidPrice);
        }
        Ok(())
    }
}
```

#### 5. CORS Configuration

```rust
use actix_cors::Cors;

// In App::new()
.wrap(
    Cors::permissive()  // For development
    // Or use restrictive for production:
    // Cors::default()
    //     .allowed_origin("https://dashboard.example.com")
    //     .allowed_methods(vec!["GET", "POST"])
    //     .allowed_headers(vec![http::header::AUTHORIZATION])
)
```

### Security Best Practices

1. **Never commit secrets** to version control
2. **Use environment variables** for sensitive data
3. **Enable TLS** in production
4. **Implement authentication** for all endpoints
5. **Validate all inputs** from untrusted sources
6. **Use least privilege** for Docker containers
7. **Keep dependencies updated**
8. **Regular security audits**
9. **Monitor for suspicious activity**
10. **Implement fail-safe mechanisms**
11. **Replace `.unwrap()` with proper error handling**
12. **Use `0.0.0.0` for containerized HOST, `127.0.0.1` for local dev**

---

## Known Issues & Limitations

### Current Limitations

1. **Symbol Pool Exhaustion**
   - Issue: Only 8 symbols available, 9th worker gets no assignment
   - Impact: System doesn't scale beyond 8 workers
   - Workaround: Add more symbols to stocklist
   - Fix: Implement symbol recycling or dynamic symbol management

2. **Stale Dashboard Connections**
   - Issue: Closed sessions not removed from `dashboard_clients`
   - Impact: Memory leak, unnecessary broadcast attempts
   - Workaround: Periodic server restart
   - Fix: Implement cleanup on disconnect and periodic health checks

3. **No Connection Limits**
   - Issue: Unlimited connections can be established
   - Impact: Potential DoS vulnerability
   - Workaround: Use firewall rules
   - Fix: Implement max connection limit per IP

4. **No Error Recovery for Workers**
   - Issue: Workers reconnect but don't handle all error scenarios
   - Impact: Workers may get stuck in bad states
   - Workaround: Manual restart
   - Fix: Implement comprehensive error handling and state recovery

5. **No Persistence**
   - Issue: All data lost on server restart
   - Impact: No historical data available
   - Workaround: N/A (not designed for persistence)
   - Fix: Add database for historical price data if needed

6. **Hardcoded Configuration**
   - Issue: HOST, ports, intervals hardcoded
   - Impact: Difficult to configure for different environments
   - Workaround: Manual code changes
   - Fix: Use environment variables or config files

7. **Dashboard WebSocket URL Hardcoded**
   - Issue: `ws://localhost:8080/ws` hardcoded in HTML
   - Impact: Dashboard won't work in production without modification
   - Workaround: Edit HTML file for production
   - Fix: Use template system or dynamic configuration

8. **Inconsistent Async Spawning**
   - Issue: Server uses `rt::spawn`, workers use `tokio::spawn`
   - Impact: Confusing for contributors, inconsistent patterns
   - Workaround: N/A (works correctly)
   - Fix: Standardize on one approach

### Technical Debt

1. **Limited Testing**
   - Only unit tests for JSON parsing
   - No integration tests, no end-to-end tests

2. **No Monitoring**
   - No metrics collection, no alerting
   - Difficult to diagnose production issues

3. **Error Handling**
   - Extensive `.unwrap()` usage throughout codebase
   - Panics instead of graceful error handling

4. **Documentation Gaps**
   - Inline code comments minimal
   - API documentation incomplete

---

## Future Improvements

### Short-term (1-3 months)

1. **Configuration Management**
   ```toml
   # config.toml
   [server]
   host = "0.0.0.0"
   port = 8080

   [symbols]
   list = ["BTCUSDT", "ETHUSDT", "BNBUSDT"]

   [workers]
   update_interval = 1  # seconds
   max_retries = 3
   ```

2. **Connection Cleanup**
   - Remove closed sessions from `dashboard_clients`
   - Implement heartbeat/ping-pong mechanism
   - Add connection timeout handling

3. **Enhanced Logging**
   - Structured JSON logging
   - Request ID tracing
   - Performance metrics logging

4. **Health Check Improvements**
   - Report connection counts
   - Check external API connectivity
   - Validate symbol pool status

5. **Docker/Podman Optimization**
   - Multi-stage builds for smaller images (already implemented)
   - Health check improvements
   - Resource limits configuration
   - Better log rotation

### Medium-term (3-6 months)

1. **Authentication & Authorization**
   - JWT token-based auth
   - Role-based access control (RBAC)
   - API key management

2. **Rate Limiting**
   - Per-IP rate limiting
   - Connection limiting
   - API rate limiting for Binance

3. **Monitoring & Observability**
   - Prometheus metrics export
   - Grafana dashboards
   - Distributed tracing

4. **Database Integration**
   - Historical price data storage
   - Query API for historical data
   - Data aggregation and analytics

5. **Improved Dashboard**
   - Charts and graphs (using Chart.js or D3.js)
   - Historical data visualization
   - Alert configuration

6. **API Resilience**
   - Circuit breaker pattern
   - Exponential backoff
   - Multiple exchange support (fallback)

7. **Error Handling**
   - Replace `.unwrap()` with proper error handling
   - Implement custom error types
   - Add recovery strategies

### Long-term (6-12 months)

1. **Microservices Architecture**
   - Separate services: API gateway, worker manager, dashboard
   - Message queue (RabbitMQ/Kafka) for communication
   - Service mesh (Istio/Linkerd)

2. **High Availability**
   - Multiple server instances with load balancing
   - Database replication
   - Geographic distribution

3. **Advanced Features**
   - Price alerts and notifications
   - Technical analysis indicators
   - Trading bot integration
   - Mobile app

4. **Enterprise Features**
   - SSO integration (OAuth2, SAML)
   - Audit logging
   - Compliance reporting
   - Multi-tenancy

5. **Performance Optimization**
   - Redis caching for price data
   - WebSocket compression
   - HTTP/2 support
   - Edge deployment (Cloudflare Workers)

---

## Additional Resources

### Documentation

- [Actix-web Documentation](https://actix.rs/docs/)
- [Tokio Documentation](https://tokio.rs/)
- [Binance API Documentation](https://binance-docs.github.io/apidocs/)
- [Rust Book](https://doc.rust-lang.org/book/)
- [WebSocket Protocol RFC 6455](https://tools.ietf.org/html/rfc6455)
- [Docker Documentation](https://docs.docker.com/)
- [Podman Documentation](https://docs.podman.io/)

### Tools & Libraries

- `websocat` - WebSocket testing tool
- `cargo-watch` - Auto-reload on file changes
- `tokio-console` - Async runtime debugging
- `tracing` - Structured logging framework
- `serde` - Serialization framework

### Community & Support

- [Actix-web GitHub](https://github.com/actix/actix-web)
- [Rust Discord](https://discord.gg/rust-lang)
- [Stack Overflow - Rust](https://stackoverflow.com/questions/tagged/rust)

### Related Projects

- Similar WebSocket-based real-time systems
- Cryptocurrency monitoring dashboards
- Rust web service examples

---

## Decision Log

### Why Actix-web?

- **Performance**: One of the fastest Rust web frameworks
- **Maturity**: Stable, well-maintained, large community
- **WebSocket Support**: Native support via `actix-ws`
- **Ecosystem**: Rich middleware and integrations

### Why WebSockets?

- **Real-time**: Low-latency, bidirectional communication
- **Efficiency**: Persistent connection reduces overhead
- **Scalability**: Can handle many concurrent connections
- **Browser Support**: Native support in all modern browsers

### Why Rust?

- **Performance**: Compiled, no runtime overhead
- **Memory Safety**: No garbage collector, predictable performance
- **Concurrency**: Excellent async/await support via Tokio
- **Reliability**: Strong type system prevents many bugs

### Why Docker/Podman?

- **Consistency**: Same environment across dev, test, production
- **Isolation**: Dependencies isolated from host system
- **Portability**: Run anywhere Docker/Podman is available
- **Scalability**: Easy horizontal scaling with Compose/K8s

### Why Edition 2024?

- **Latest Features**: Access to newest Rust language features
- **Future-proof**: Current stable edition for new projects
- **Improved Tooling**: Better IDE support and error messages

### Why Multi-stage Docker Builds?

- **Image Size**: Smaller final images (only runtime dependencies)
- **Security**: No build tools or compiler in final image
- **Caching**: Faster rebuilds with dependency caching layer
- **Reproducibility**: Consistent builds across environments

---

## Code Patterns and Conventions

### Async Spawning

**Server** (`src/ws_handler.rs`):
```rust
use actix_web::rt;
rt::spawn(async move {
    // async task
});
```

**Workers** (`workers/src/main.rs`):
```rust
use tokio;
tokio::spawn(async move {
    // async task
});
```

**Note**: These are inconsistent. Prefer `tokio::spawn` for consistency.

### Error Handling

**Current Pattern** (avoid in production):
```rust
let result = some_operation();
result.unwrap()  // Will panic on error
```

**Better Pattern**:
```rust
use anyhow::Result;

async fn some_function() -> Result<()> {
    let result = some_operation()
        .context("Failed to perform operation")?;
    Ok(())
}
```

### Shared State Access

**Pattern**:
```rust
// Lock the mutex
let mut data = state.lock().unwrap();

// Perform operations
data.push(item);

// Drop lock early if needed
drop(data);

// Continue with other operations
```

### Static File Embedding

**Pattern**:
```rust
// Embed HTML at compile time
let html_content = include_str!("../static/dashboard.html");

// Serve as response
HttpResponse::Ok()
    .content_type("text/html")
    .body(html_content)
```

**CSS files** are served separately, not embedded.

### WebSocket Message Handling

**Server Pattern**:
```rust
match msg {
    Ok(AggregatedMessage::Text(text)) => {
        // Handle text message
    }
    Ok(AggregatedMessage::Ping(msg)) => {
        session.pong(&msg).await.unwrap();
    }
    _ => {}
}
```

**Worker Pattern**:
```rust
match msg {
    Ok(Message::Text(text)) => {
        // Handle text message
    }
    Ok(Message::Close(_)) => {
        // Handle close
        break;
    }
    _ => {}
}
```

### Dependency Caching in Docker

**Pattern**:
```dockerfile
# Stage 1: Builder
FROM rust:1.82-slim as builder
WORKDIR /app

# Copy only dependency files first
COPY Cargo.toml Cargo.lock ./

# Create dummy main.rs to cache dependencies
RUN mkdir src && echo "fn main() {}" > src/main.rs
RUN cargo build --release
RUN rm -rf src

# Now copy actual source and rebuild
COPY src ./src
RUN touch src/main.rs && cargo build --release
```

This caches dependencies between builds, speeding up rebuilds when only source code changes.

---

## Important Gotchas

1. **HOST Configuration**
   - Local dev: `127.0.0.1:8080`
   - Containers: `0.0.0.0:8080`
   - Must update in code AND docker-compose.yml

2. **Worker Binary Name**
   - Package name in `workers/Cargo.toml`: `b0t`
   - Binary name: `b0t`
   - Don't search for "worker" binary

3. **Podman vs Docker**
   - Podman Compose doesn't support `deploy.replicas`
   - Use `--scale workers=N` flag with Podman
   - Otherwise similar syntax

4. **Symbol Pool**
   - Limited to 8 symbols by default
   - Each worker consumes one symbol
   - 9th worker gets no assignment
   - Symbols never recycled (current implementation)

5. **Dashboard Connection Cleanup**
   - Closed dashboard connections NOT removed from list
   - Memory leak over time
   - Manual restart required (or implement cleanup)

6. **Static File Serving**
   - HTML: Embedded via `include_str!` at compile time
   - CSS: Served as static file from `/static/`
   - Changes to CSS require server restart or container rebuild

7. **WebSocket URL in Dashboard**
   - Hardcoded to `ws://localhost:8080/ws`
   - Must edit HTML for production deployment
   - Consider template system for flexibility

8. **Testing Coverage**
   - Only unit tests for JSON parsing
   - No integration tests
   - Manual testing required for WebSocket flows

9. **Error Handling**
   - Extensive `.unwrap()` usage
   - Panics on errors instead of graceful handling
   - Not suitable for production use without changes

10. **Async Spawning Inconsistency**
    - Server uses `rt::spawn`
    - Workers use `tokio::spawn`
    - Both work but confusing for contributors

---

## Conclusion

This AGENTS.md provides comprehensive guidance for working with the Fin-Dashboard codebase. Key takeaways:

1. **Understand the architecture** before making changes
2. **Follow the existing patterns** for consistency
3. **Test thoroughly** before deploying
4. **Monitor system health** in production
5. **Plan for scalability** from the start
6. **Prioritize security** in all modifications
7. **Document changes** for future maintainers
8. **Be aware of gotchas** and limitations

For day-to-day development, refer to:
- `CLAUDE.md` - Quick reference and common tasks
- `DOCKER.md` - Docker/Podman-specific guidance
- This `AGENTS.md` - Deep technical context and best practices

Remember: This is a distributed real-time system. Changes can have cascading effects across components. Always consider the full system impact before implementing modifications.

---

*Last Updated: 2025*
*Maintained by: Fin-Dashboard Team*
*For questions or updates, contact the development team*
