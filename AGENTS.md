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
3. **Scalability**: Horizontal scaling of workers via Docker
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
| `tokio` | 1.0 | Async runtime |
| `reqwest` | 0.11 | HTTP client for Binance API |
| `serde` | 1.0 | Serialization |
| `serde_json` | 1.0 | JSON handling |
| `url` | 2.4 | URL parsing |
| `clap` | 4.x | Command-line argument parsing |

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
│   └── Cargo.toml           # Worker dependencies
├── static/
│   └── dashboard.html       # Dashboard UI (embedded in binary)
├── Cargo.toml               # Server dependencies
├── Dockerfile               # Server container build
├── docker-compose.yml       # Multi-container orchestration
├── run_all.sh               # Start server + workers locally
├── docker-build.sh          # Build Docker images
├── CLAUDE.md                # Developer guidance
├── DOCKER.md                # Docker documentation
└── README.md                # Project overview
```

### File Responsibilities

#### `src/main.rs`
- HTTP server configuration
- Route definitions: `/`, `/health`, `/dashboard`, `/ws`
- Shared state initialization
- Application data setup with `web::Data::new()`

#### `src/ws_handler.rs`
- WebSocket connection handling
- User authentication (bot vs dashboard)
- Symbol assignment to workers
- Price message parsing and broadcasting
- Session management

#### `workers/src/main.rs`
- WebSocket client implementation
- Binance API integration
- Price fetching interval logic (1s)
- Auto-reconnection on failure
- CLI argument support (via clap)

#### `static/dashboard.html`
- WebSocket client implementation (JavaScript)
- Real-time price display
- Connection status indicators

---

## Development Workflow

### Local Development Setup

1. **Prerequisites**
   - Rust toolchain (edition 2024)
   - `websocat` for WebSocket testing
   - Optional: Docker & Docker Compose

2. **Start Development Server**
   ```bash
   # Terminal 1: Start server
   cargo run
   
   # Terminal 2: Start worker
   cd workers && cargo run
   
   # Repeat for additional workers in separate terminals
   ```

3. **Quick Start Script**
   ```bash
   ./run_all.sh  # Starts server + 5 workers
   ```

4. **Access Dashboard**
   - Open browser: `http://localhost:8080/dashboard`
   - WebSocket endpoint: `ws://127.0.0.1:8080/ws`

### Code Modification Workflow

1. **Modify Server Code**
   ```bash
   # Edit src/main.rs or src/ws_handler.rs
   cargo run  # Auto-reload not configured, manual restart needed
   ```

2. **Modify Worker Code**
   ```bash
   # Edit workers/src/main.rs
   cd workers && cargo run
   ```

3. **Run Tests**
   ```bash
   cargo test                    # Run all tests
   cargo test ws_handler         # Run specific module
   cargo test -- --nocapture     # Show test output
   ```

### Docker Development Workflow

1. **Build Images**
   ```bash
   ./docker-build.sh  # Build both images
   # Or manually:
   docker-compose build
   ```

2. **Start Services**
   ```bash
   docker-compose up -d
   ```

3. **View Logs**
   ```bash
   docker-compose logs -f      # All services
   docker-compose logs -f web  # Server only
   docker-compose logs -f workers  # Workers only
   ```

4. **Scale Workers**
   ```bash
   docker-compose up -d --scale workers=10
   ```

5. **Stop Services**
   ```bash
   docker-compose down
   ```

### Making Changes

**When modifying the codebase, follow this order:**

1. Identify the affected component (server/worker)
2. Make changes to source code
3. Run relevant tests
4. Build Docker images if Docker is being used
5. Restart affected services
6. Verify functionality

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

### Task 2: Change Price Update Interval

**Location**: `workers/src/main.rs` (interval configuration)

Find the interval timer configuration and adjust:
```rust
// Change from 1 second to desired interval
let mut interval = time::interval(Duration::from_secs(1));  // Current: 1s
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
const HOST: &str = "127.0.0.1:8080";  // Change to desired port
```

**Also update**: `docker-compose.yml` port mapping, `DOCKER.md` documentation

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

Add cleanup logic when dashboard disconnects:
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

**Location**: `static/dashboard.html`

Modify HTML/JavaScript to:
- Change layout/styling
- Add charts/graphs
- Implement historical data display
- Add alerts/notifications

### Task 9: Change API Endpoint

**Location**: `workers/src/main.rs` (fetch_price function)

```rust
const API_URL: &str = "https://api.binance.com/api/v3/ticker/price";
// Change to different exchange or custom API
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
```

### Writing New Tests

**Principles**:
1. Test public APIs and critical business logic
2. Mock external dependencies (Binance API)
3. Test edge cases (empty inputs, invalid formats)
4. Test concurrent access patterns for shared state

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
1. Start server: `cargo run`
2. Start multiple workers in separate terminals
3. Open dashboard in browser
4. Verify real-time updates
5. Test reconnection: kill workers, verify they reconnect
6. Test broadcast: verify all dashboards receive updates

**WebSocket Manual Testing**:
```bash
# Install websocat: brew install websocat
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
- Verify network connectivity in Docker environment

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
# or
cargo install websocat

# Test connection
websocat ws://127.0.0.1:8080/ws
```

**HTTP Testing**:
```bash
# Health check
curl http://127.0.0.1:8080/health

# Dashboard endpoint
curl http://127.0.0.1:8080/dashboard
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

### Optimization Strategies

**Server-Side**:
```rust
// 1. Use Tokio channels for broadcasting
use tokio::sync::broadcast;

// 2. Batch messages if possible
// 3. Implement connection pooling
// 4. Use async DNS resolution
```

**Worker-Side**:
```rust
// 1. Implement exponential backoff for API calls
// 2. Cache responses if data doesn't change frequently
// 3. Use connection pooling for HTTP client
```

### Capacity Planning

**Current Limits**:
- 8 symbols (expandable)
- 1-second update interval
- 5 workers (default)

**Scaling Considerations**:
- Horizontal scaling: Add more workers
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

### Production Checklist

- [ ] Change default port (8080 → production port)
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
RUST_LOG=info                    # Log level
HOST=0.0.0.0                     # Listen address
PORT=8080                        # Listen port
```

**Workers**:
```bash
RUST_LOG=info                    # Log level
SERVER_URL=ws://server:8080/ws   # WebSocket URL
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
    restart: always              # Always restart on failure
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
- Docker Compose files

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

### Technical Debt

1. **Hardcoded Configuration**
   - Stock symbols, ports, intervals hardcoded
   - Should use configuration files or environment variables

2. **Limited Testing**
   - Only unit tests for JSON parsing
   - No integration tests, no end-to-end tests

3. **No Monitoring**
   - No metrics collection, no alerting
   - Difficult to diagnose production issues

4. **Synchronous Error Handling**
   - Some error scenarios could benefit from async recovery

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

3. **Enhanced Logging**
   - Structured JSON logging
   - Request ID tracing
   - Performance metrics logging

4. **Health Check Improvements**
   - Report connection counts
   - Check external API connectivity
   - Validate symbol pool status

5. **Docker Optimization**
   - Multi-stage builds for smaller images
   - Health check improvements
   - Resource limits configuration

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

### Why Docker?

- **Consistency**: Same environment across dev, test, production
- **Isolation**: Dependencies isolated from host system
- **Portability**: Run anywhere Docker is available
- **Scalability**: Easy horizontal scaling with Compose/K8s

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

For day-to-day development, refer to:
- `CLAUDE.md` - Quick reference and common tasks
- `DOCKER.md` - Docker-specific guidance
- This `AGENTS.md` - Deep technical context and best practices

Remember: This is a distributed real-time system. Changes can have cascading effects across components. Always consider the full system impact before implementing modifications.

---

*Last Updated: 2024*
*Maintained by: Fin-Dashboard Team*
*For questions or updates, contact the development team*
