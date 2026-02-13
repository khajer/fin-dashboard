# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

Fin-Dashboard is a real-time cryptocurrency price monitoring system built with Rust. It consists of:
- **Web Server**: Actix-web server with WebSocket support (port 8080)
- **Worker Bots**: Multiple bot instances that fetch price data from Binance API
- **Dashboard UI**: Real-time web interface for viewing price updates

## Development Commands

### Building and Running

```bash
# Run the web server (from project root)
cargo run

# Run a single worker bot
cd workers && cargo run

# Run server + 5 workers simultaneously
./run_all.sh

# Build release binaries
cargo build --release
cd workers && cargo build --release
```

### Testing

```bash
# Run all tests
cargo test

# Run tests for specific module
cargo test ws_handler

# Run tests with output
cargo test -- --nocapture

# Test WebSocket connection manually
websocat ws://127.0.0.1:8080/ws
```

### Docker

```bash
# Build and start all services
docker-compose up -d

# View logs
docker-compose logs -f

# Scale workers
docker-compose up -d --scale workers=10

# Stop all services
docker-compose down
```

## Architecture

### Communication Flow

1. **Server Startup**: Server initializes with a stock symbol list (BTCUSDT, ETHUSDT, etc.) and empty dashboard clients list
2. **Worker Login**: Worker connects via WebSocket, sends `{"username": "bot"}`, receives assigned symbol
3. **Dashboard Login**: Dashboard connects, sends `{"username": "dashboard"}`, gets added to broadcast list
4. **Price Updates**: Workers fetch prices from Binance API every 1 second, send `{"symbol": "BTCUSDT", "price": "42000.00"}` to server
5. **Broadcast**: Server broadcasts price updates to all connected dashboard clients

### Key Components

**Server (`src/`)**:
- `main.rs`: HTTP server setup, routes (`/`, `/health`, `/dashboard`, `/ws`), shared state (stocklist, dashboard_clients)
- `ws_handler.rs`: WebSocket handler with login logic and message routing
  - `parse_login_text()`: Authenticates users (bot/dashboard), assigns symbols
  - `parse_command()`: Parses price data and broadcasts to dashboards
  - Note: Uses Arc<Mutex<>> for thread-safe shared state

**Workers (`workers/src/`)**:
- `main.rs`: WebSocket client that logs in, receives symbol assignment, fetches prices
  - `fetch_price()`: Calls Binance API endpoint
  - `interval_func()`: Sends price updates every 1 second
  - Auto-reconnects on connection failure

**Frontend (`static/`)**:
- `dashboard.html`: WebSocket client UI
- `dashboard.css`: Dashboard styling

### State Management

The server maintains two critical shared state structures:
- `Arc<Mutex<Vec<&'static str>>>`: Stock symbol pool (symbols are removed when assigned)
- `Arc<Mutex<Vec<Session>>>`: Connected dashboard WebSocket sessions for broadcasting

### WebSocket Message Formats

**Login (bot)**: `{"username": "bot"}`
**Login (dashboard)**: `{"username": "dashboard"}`
**Login Response**: `{"status": "success", "cmd": "BTCUSDT"}`
**Price Update**: `{"symbol": "BTCUSDT", "price": "42000.50"}`

## Code Conventions

- Use `tracing` crate for logging (not `println!` in server code)
- WebSocket messages are JSON serialized using `serde_json`
- Server edition: 2024, uses Actix-web 4.x and actix-ws 0.3.x
- Workers use tokio-tungstenite for WebSocket client
- Error handling: Workers reconnect on failure, server logs errors

## Testing Notes

- Tests are in `src/ws_handler.rs` (lines 138-249)
- Focus on JSON parsing, state management, and message format validation
- No integration tests for WebSocket connections currently

## Common Pitfalls

- Symbol pool exhaustion: If more than 8 workers connect, no symbols remain (add more to stocklist in main.rs:58-67)
- Dashboard clients list grows unbounded: Closed connections aren't removed from dashboard_clients
- Worker reconnection uses 1-second delay, may overwhelm server with many workers
