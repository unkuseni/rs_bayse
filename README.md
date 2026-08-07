# rs_bayse — Bayse Markets API bindings for Rust

[![Crates.io](https://img.shields.io/crates/v/rs_bayse)](https://crates.io/crates/rs_bayse)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)

**rs_bayse** is a Rust client library for the [Bayse Markets](https://bayse.markets) API — the culturally-native prediction market platform for Africa. It provides full coverage of the REST API and WebSocket feeds with ergonomic Rust types.

## Table of Contents

- [Features](#features)
- [Quick Start](#quick-start)
- [Prerequisites](#prerequisites)
- [Installation](#installation)
- [Configuration](#configuration)
- [Friendly API](#friendly-api)
- [Usage](#usage)
  - [REST API (Public)](#rest-api-public)
  - [REST API (Authenticated)](#rest-api-authenticated)
  - [WebSocket (Market Data)](#websocket-market-data)
  - [WebSocket (User Orders)](#websocket-user-orders)
- [Architecture](#architecture)
- [Examples](#examples)
- [Running Tests](#running-tests)
- [Development](#development)
- [Contributing](#contributing)
- [License](#license)

## Features

### REST API
- **System** — Health check, version info
- **User** — Lookup, login, API key management (create, list, revoke, rotate)
- **Trading** — Events, quotes, order placement (single & batch), portfolio, PnL, orders, mint/burn shares, activities, sports markets (games, leagues, teams)
- **Wallet** — Asset balances
- **Market Data** — Price history, order book, ticker, trades
- **Market Makers** — Liquidity rewards & maker rebates

### WebSocket API
- **Market Data** (`/ws/v1/markets`) — Real-time activity feeds, price updates, order book snapshots
- **User Orders** (`/ws/v1/user`) — Real-time fill updates for authenticated users
- **Asset Prices** (`/ws/v1/realtime`) — Live crypto and FX price feeds

### Authentication
- **Public** — No authentication required
- **Session** — `x-auth-token` + `x-device-id` headers
- **Read** — `X-Public-Key` header
- **Write** — `X-Public-Key` + `X-Timestamp` + `X-Signature` (HMAC-SHA256)

## Quick Start

```rust
use bayse::prelude::*;

#[tokio::main]
async fn main() -> Result<(), BayseError> {
    // Public endpoints (no auth needed)
    let sys = SystemManager::new(None, None);
    let healthy = sys.health().await?;
    println!("API healthy: {healthy}");

    // Authenticated endpoints (with API key)
    let trading = TradingManager::new(
        Some("your_public_key".into()),
        Some("your_secret_key".into()),
    );
    let events = trading.list_events(Some(1), Some(20)).await?;
    println!("Events: {events:#?}");

    Ok(())
}
```

## Friendly API

If you'd rather skip the boilerplate, use the [`BayseClient`] facade. It
bundles every manager into one object and returns **typed data** instead of
raw JSON — no digging through `serde_json::Value`.

```rust
use bayse::prelude::*;

#[tokio::main]
async fn main() -> Result<(), BayseError> {
    // Public access — no credentials needed
    let client = BayseClient::public();

    // Typed events (no pagination wrapper to unwrap)
    let events = client.open_events(Some(5)).await?;
    println!("{} open events", events.len());

    // Typed ticker, order book, trades, price history
    let ticker = client
        .market_data
        .get_ticker("market_id", Some("YES"), None)
        .await?;
    println!("last: {} spread: {}", ticker.last_price, ticker.spread);

    let books = client
        .market_data
        .get_order_book(&["outcome_id"], Some(5), None)
        .await?;
    for book in &books {
        println!(
            "{} — best bid {:?}, best ask {:?}",
            book.market_id,
            book.bids.first(),
            book.asks.first()
        );
    }

    // One-call onboarding: login + create API key + wire everything
    let (client, key) = BayseClient::login_and_create_api_key(
        "you@example.com",
        "your-password",
        "my-trading-bot",
    )
    .await?;
    println!("public key: {}", key.public_key);
    println!("secret key: {} (store this!)", key.secret_key);

    // Wallet balances as typed assets
    let balances = client.balances().await?;
    for asset in &balances {
        println!(
            "{} — available {}",
            asset.symbol, asset.available_balance
        );
    }

    Ok(())
}
```

**What you get for free:**

- `BayseClient::public()` / `with_api_key()` / `with_session()` — one object
  with every manager wired up.
- `BayseClient::login()` / `login_and_create_api_key()` — the multi-step
  session + API key flows in a single call.
- `open_events()`, `markets_for()`, `balances()` — combine steps and strip
  response wrappers.
- Typed market data: [`Ticker`], [`OrderBook`], [`Trade`], [`PricePoint`],
  [`UserProfile`], [`EventSeries`], [`SeriesEventSummary`], sports
  ([`SportsGame`], [`SportsTeam`], [`SportsLeague`]), and [`Activity`].

Run the full walkthrough with:

```bash
cargo run --example friendly
```

## Prerequisites

- **Rust** 1.75+ (edition 2021)
- **Tokio** runtime (for async operations)
- A Bayse Markets account and API credentials (for authenticated endpoints)

## Installation

Add to your `Cargo.toml`:

```toml
[dependencies]
rs_bayse = "0.2.0"
tokio = { version = "1", features = ["full"] }
```

## Configuration

### Environment Variables

| Variable | Description | Required |
|---|---|---|
| `BAYSE_API_KEY` | Your public API key | For authenticated endpoints |
| `BAYSE_API_SECRET` | Your secret API key | For write-level endpoints |

### Custom Configuration

```rust
use bayse::prelude::*;

let config = Config::default()
    .with_api_key("pub_key".into(), "sec_key".into())
    .with_session_token("token".into(), "device_id".into());

let trading = TradingManager::new_with_config(config);
```

## Usage

### REST API (Public)

```rust
use bayse::prelude::*;

#[tokio::main]
async fn main() -> Result<(), BayseError> {
    let sys = SystemManager::new(None, None);
    let market_data = MarketDataManager::new(None, None);

    // Health check
    println!("Healthy: {}", sys.health().await?);

    // Version info
    println!("Version: {:?}", sys.version().await?);

    // Price history
    let prices = market_data
        .get_price_history("event_id", Some("24H"), None, None)
        .await?;
    println!("Prices: {prices:#?}");

    // Order book (CLOB markets)
    let ob = market_data
        .get_order_book(&["outcome_id"], Some(10), None)
        .await?;
    println!("Order book: {ob:#?}");

    // Ticker
    let ticker = market_data
        .get_ticker("market_id", Some("YES"), None)
        .await?;
    println!("Ticker: {ticker:#?}");

    // Recent trades
    let trades = market_data
        .get_trades(&TradesQuery {
            market_id: Some("market_id".into()),
            size: Some(50),
            ..Default::default()
        })
        .await?;
    println!("Trades: {trades:#?}");

    Ok(())
}
```

### REST API (Authenticated)

```rust
use bayse::prelude::*;

#[tokio::main]
async fn main() -> Result<(), BayseError> {
    let api_key = std::env::var("BAYSE_API_KEY").expect("BAYSE_API_KEY required");
    let api_secret = std::env::var("BAYSE_API_SECRET").expect("BAYSE_API_SECRET required");

    let trading = TradingManager::new(Some(api_key.clone()), Some(api_secret));
    let wallet = WalletManager::new(Some(api_key), None);

    // List events
    let events = trading.list_events(Some(1), Some(20)).await?;
    println!("Events: {events:#?}");

    // Get portfolio
    let portfolio = trading.get_portfolio().await?;
    println!("Portfolio: {portfolio:#?}");

    // Get wallet assets
    let assets = wallet.get_assets().await?;
    println!("Assets: {assets:#?}");

    // Place an order
    let order = trading
        .place_order(
            "event_id",
            "market_id",
            &PlaceOrderRequest {
                side: "BUY".into(),
                outcome_id: "outcome_id".into(),
                amount: 100.0,
                order_type: "LIMIT".into(),
                currency: Some("USD".into()),
                price: Some(0.55),
                time_in_force: Some("GTC".into()),
                post_only: None,
                stp_mode: None,
                max_slippage: None,
                expires_at: None,
            },
        )
        .await?;
    println!("Order: {order:#?}");

    Ok(())
}
```

### WebSocket (Market Data)

```rust
use bayse::prelude::*;

#[tokio::main]
async fn main() {
    let ws: Stream = Bayse::new(None, None);

    let sub = WsSubscription::new("subscribe", "prices")
        .with_event_id("your_event_id".into());

    ws.subscribe_market(sub, |msg| {
        println!("Market update: {msg:#?}");
        Ok::<_, std::io::Error>(())
    })
    .await
    .ok();
}
```

### WebSocket (User Orders)

```rust
use bayse::prelude::*;

#[tokio::main]
async fn main() {
    let api_key = std::env::var("BAYSE_API_KEY").unwrap();
    let ws: Stream = Bayse::new(Some(api_key.clone()), None);

    // Every message on the user endpoint must carry auth credentials.
    let sub = WsSubscription::new("subscribe", "orders")
        .with_market_ids(vec!["market_id".into()])
        .with_auth(WsAuth::with_api_key(api_key));

    ws.subscribe_user(sub, |msg| {
        println!("Order update: {msg:#?}");
        Ok::<_, std::io::Error>(())
    })
    .await
    .ok();
}
```

## Architecture

The library is organised around **manager types**, each responsible for a domain area:

| Manager | Domain | Auth Level |
|---|---|---|
| `SystemManager` | Health check, version | Public |
| `UserManager` | Login, API keys | Session / Public |
| `TradingManager` | Events, orders, portfolio, PnL | Read / Write |
| `WalletManager` | Asset balances | Read |
| `MarketDataManager` | Price history, order book, ticker, trades | Public |
| `MarketMakerManager` | Liquidity rewards, maker rebates | Read |
| `Stream` | WebSocket feeds | Public / Per-message |

### Manager Types

Every manager struct implements the `Bayse` trait:

```rust
pub trait Bayse {
    fn new(api_key: Option<String>, secret_key: Option<String>) -> Self;
    fn new_with_config(config: Config) -> Self;
}
```

## Examples

Check the [examples](examples/) directory:

- [`rest_api.rs`](examples/rest_api.rs) — Demonstrates public and authenticated REST endpoints
- [`friendly.rs`](examples/friendly.rs) — Demonstrates the `BayseClient` facade with typed data helpers
- [`websocket.rs`](examples/websocket.rs) — Demonstrates WebSocket subscriptions

```bash
# Run REST API examples
cargo run --example rest_api

# Run the friendly facade example
cargo run --example friendly

# Run WebSocket examples
cargo run --example websocket market
```

## Running Tests

```bash
# Run all tests
cargo test

# Run tests with logging
RUST_LOG=debug cargo test

# Run specific test
cargo test test_name
```

## Development

### Pre-commit Hooks

```bash
pip install pre-commit
pre-commit install
```

### CI

The project uses GitHub Actions for CI. Configuration is in `.github/workflows`.

## Contributing

Contributions are welcome! Please follow these guidelines:

1. **File an issue** — Discuss your proposed change before implementing.
2. **Fork the repo** — Create a feature branch.
3. **Write tests** — Cover new functionality.
4. **Run `cargo fmt` and `cargo clippy`** — Ensure code quality.
5. **Submit a PR** — Reference the issue.

### Code Style

- Follow Rust standard formatting (`cargo fmt`).
- Run `cargo clippy -- -W clippy::all -W clippy::pedantic` and fix warnings.
- Document all public items with doc comments.
- Use `tracing` crate for structured logging.

## License

This project is licensed under the MIT License — see the [LICENSE](LICENSE) file for details.

## Acknowledgments

### Credit

This library is inspired by the excellent `rs_bybit` crate (Bybit V5 API bindings) and follows similar architectural patterns.

### Contact

- Twitter: [@unkuseni](https://twitter.com/unkuseni)
- GitHub: [unkuseni](https://github.com/unkuseni)
