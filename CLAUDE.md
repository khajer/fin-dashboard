# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## What this is

A real-time crypto price dashboard: an actix-web WebSocket server (this crate) that
relays prices to browser clients, fed by a separate bot process (`workers/`) that
polls Binance's REST API and forwards prices over WebSocket. Server defaults to
`127.0.0.1:8080`.

## Repo layout

This is **two independent Cargo crates**, not a workspace — each has its own
`Cargo.toml`/`Cargo.lock`/`target/` and must be built/run from its own directory.

- `/` (`fin-dashboard`) — the actix-web server: `src/main.rs` (routes, app state),
  `src/ws_handler.rs` (WebSocket protocol + relay logic), `static/dashboard.html`
  (browser client, plain JS/WebSocket, no build step).
- `workers/` (`workers`) — the price-fetching bot: `workers/src/main.rs`.

## Commands

Run from the relevant crate directory (`.` for the server, `workers/` for the bot).

```sh
cargo build
cargo run
cargo test                    # server crate only; workers has no tests
cargo test test_name          # run a single test
cargo fmt
cargo clippy
```

Manual WebSocket testing:
```sh
websocat ws://127.0.0.1:8080/ws
```

## Architecture: the `/ws` protocol

Both dashboard browser clients and worker bots connect to the **same** `/ws`
endpoint; the server tells them apart by the first message's `username` field
(handled in `ws_handler::parse_login_text`):

1. Client sends `{"username": "bot"}` or `{"username": "dashboard"}`.
2. Server pops one symbol off the shared `stocklist` (`Arc<Mutex<Vec<&str>>>` app
   state, seeded in `main.rs`) and replies `{"status": "success", "cmd": "<SYMBOL>"}`.
   - A `bot` client uses `cmd` as the symbol it should start polling Binance for.
   - A `dashboard` client is registered into `dashboard_clients`
     (`Arc<Mutex<Vec<Session>>>` app state) so it receives broadcast price updates.
   - Each connection consumes one symbol from `stocklist`; once empty, further
     logins get no symbol assigned.
3. After login, a bot sends `{"symbol": "...", "price": "..."}` messages
   (`parse_command` in `ws_handler.rs`), which the server fans out to every
   session in `dashboard_clients`.

Login parsing failure falls through to `parse_command`, i.e. any non-login JSON
on the socket is treated as a price update to broadcast — there's no separate
"logged in" state tracked per-connection.

The worker (`workers/src/main.rs`) mirrors this: connect → send `{"username":
"bot"}` → on success, spawn a loop that polls
`https://api.binance.com/api/v3/ticker/price?symbol=<SYMBOL>` every second and
sends the result back over the same socket. It reconnects on any connection
failure.
