//! # REST API Usage Examples
//!
//! Demonstrates how to use `rs_bayse` to interact with the Bayse Markets REST API.
//!
//! ## Setup
//!
//! ```bash
//! export BAYSE_API_KEY="your_public_key"
//! export BAYSE_API_SECRET="your_secret_key"
//! ```
//!
//! ## Running
//!
//! ```bash
//! # Public endpoints only (no API keys needed)
//! cargo run --example rest_api
//!
//! # With API keys for authenticated endpoints
//! BAYSE_API_KEY="your_key" BAYSE_API_SECRET="your_secret" cargo run --example rest_api
//! ```

use bayse::prelude::*;
use std::env;

/// Retrieves API credentials from environment variables.
fn get_credentials() -> (Option<String>, Option<String>) {
    let api_key = env::var("BAYSE_API_KEY").ok();
    let api_secret = env::var("BAYSE_API_SECRET").ok();
    (api_key, api_secret)
}

/// Demonstrates public endpoints.
async fn run_public_examples() -> Result<(), BayseError> {
    println!("\n═══════════════════════════════════════");
    println!("  PUBLIC ENDPOINTS");
    println!("═══════════════════════════════════════\n");

    let sys = SystemManager::new(None, None);

    // --- Health Check ---
    println!("▶ Health Check:");
    match sys.health().await {
        Ok(health) => println!("  Status: {}", health.status),
        Err(e) => println!("  ✗ Failed: {e}"),
    }

    // --- Version ---
    println!("\n▶ Version:");
    match sys.version().await {
        Ok(v) => println!("  Version: {v:#?}"),
        Err(e) => println!("  Error: {e}"),
    }

    // --- Market Data: Price History ---
    println!("\n▶ Price History (example event):");
    let market_data = MarketDataManager::new(None, None);
    match market_data
        .get_price_history("example_event_id", None, None, None)
        .await
    {
        Ok(data) => println!("  Price history: {data:#?}"),
        Err(e) => println!("  Error: {e}"),
    }

    // --- Market Data: Order Book ---
    println!("\n▶ Order Book:");
    match market_data.get_order_book(&["market_id_1"], Some(5)).await {
        Ok(book) => println!("  Order book: {book:#?}"),
        Err(e) => println!("  Error: {e}"),
    }

    // --- Market Data: Ticker ---
    println!("\n▶ Ticker:");
    match market_data.get_ticker("example_market_id").await {
        Ok(ticker) => println!("  Ticker: {ticker:#?}"),
        Err(e) => println!("  Error: {e}"),
    }

    // --- Market Data: Trades ---
    println!("\n▶ Recent Trades:");
    match market_data
        .get_trades(Some(&["market_id_1"]), Some(10))
        .await
    {
        Ok(trades) => println!("  Trades: {trades:#?}"),
        Err(e) => println!("  Error: {e}"),
    }

    Ok(())
}

/// Demonstrates authenticated endpoints.
async fn run_authenticated_examples(api_key: &str, api_secret: &str) -> Result<(), BayseError> {
    println!("\n═══════════════════════════════════════");
    println!("  AUTHENTICATED ENDPOINTS");
    println!("═══════════════════════════════════════\n");

    let trading = TradingManager::new(Some(api_key.into()), Some(api_secret.into()));
    let wallet = WalletManager::new(Some(api_key.into()), None);

    // --- List Events ---
    println!("▶ List Events:");
    match trading.list_events(Some(1), Some(10)).await {
        Ok(events) => println!("  Events: {events:#?}"),
        Err(e) => println!("  Error: {e}"),
    }

    // --- Portfolio ---
    println!("\n▶ Portfolio:");
    match trading.get_portfolio().await {
        Ok(portfolio) => println!("  Portfolio: {portfolio:#?}"),
        Err(e) => println!("  Error: {e}"),
    }

    // --- Wallet Assets ---
    println!("\n▶ Wallet Assets:");
    match wallet.get_assets().await {
        Ok(assets) => println!("  Assets: {assets:#?}"),
        Err(e) => println!("  Error: {e}"),
    }

    // --- List Orders ---
    println!("\n▶ List Orders:");
    match trading
        .list_orders(&ListOrdersQuery {
            page: Some(1),
            size: Some(10),
            ..Default::default()
        })
        .await
    {
        Ok(orders) => println!("  Orders: {orders:#?}"),
        Err(e) => println!("  Error: {e}"),
    }

    Ok(())
}

#[tokio::main]
async fn main() -> Result<(), BayseError> {
    println!("═══════════════════════════════════════");
    println!(
        "  rs_bayse v{} — REST API Examples",
        env!("CARGO_PKG_VERSION")
    );
    println!("═══════════════════════════════════════");

    // Public endpoints (no auth needed)
    run_public_examples().await?;

    // Authenticated endpoints
    let (api_key, api_secret) = get_credentials();
    match (api_key, api_secret) {
        (Some(key), Some(secret)) if !key.is_empty() && !secret.is_empty() => {
            run_authenticated_examples(&key, &secret).await?;
        }
        _ => {
            println!(
                "\n⚠  Skipping authenticated examples — set BAYSE_API_KEY and BAYSE_API_SECRET"
            );
        }
    }

    println!("\n═══════════════════════════════════════");
    println!("  All examples completed.");
    println!("═══════════════════════════════════════\n");

    Ok(())
}
