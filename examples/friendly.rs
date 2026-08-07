//! # Friendly API Examples
//!
//! Demonstrates the high-level [`BayseClient`] facade and the typed
//! market-data helpers — no raw `serde_json::Value` digging required.
//!
//! ## Running
//!
//! ```bash
//! # Public endpoints only
//! cargo run --example friendly
//!
//! # Full onboarding (login + create API key) with real credentials
//! BAYSE_EMAIL="you@example.com" BAYSE_PASSWORD="your-password" cargo run --example friendly
//! ```

use bayse::prelude::*;
use std::env;

#[tokio::main]
async fn main() -> Result<(), BayseError> {
    println!("═══════════════════════════════════════");
    println!("  rs_bayse v{} — Friendly API Examples", env!("CARGO_PKG_VERSION"));
    println!("═══════════════════════════════════════\n");

    // ------------------------------------------------------------------
    // 1. Public client — browse events and market data without auth
    // ------------------------------------------------------------------
    let client = BayseClient::public();

    // One call instead of manager + list_events + unwrap the wrapper
    let events = client.open_events(Some(5)).await?;
    println!("▶ Open events ({}):", events.len());
    for event in &events {
        println!(
            "  · {} [{}] — {} markets (engine: {})",
            event.title,
            event.category,
            event.markets.len(),
            event.engine,
        );
    }

    // Typed market data — no JSON digging
    if let Some(first) = events.first() {
        if let Some(market) = first.markets.first() {
            let ticker = client
                .market_data
                .get_ticker(&market.id, Some("YES"), None)
                .await;
            if let Ok(t) = ticker {
                println!(
                    "\n▶ Ticker for {}: last={} bid={} ask={} (24h vol {})",
                    t.market_id, t.last_price, t.best_bid, t.best_ask, t.volume_24h
                );
            }

            let books = client
                .market_data
                .get_order_book(&["outcome_id"], Some(3), None)
                .await;
            if let Ok(b) = books {
                println!("\n▶ Order book ({}):", b.len());
                for book in &b {
                    let best_bid = book.bids.first().map(|l| l.price).unwrap_or(0.0);
                    let best_ask = book.asks.first().map(|l| l.price).unwrap_or(0.0);
                    println!("  {} — best bid {best_bid} / best ask {best_ask}", book.market_id);
                }
            }
        }

        let history = client
            .market_data
            .get_price_history(&first.id, Some("24H"), None, None)
            .await;
        if let Ok(h) = history {
            println!("\n▶ Price history points per market:");
            for (market_id, points) in &h {
                println!("  {market_id}: {} points", points.len());
            }
        }
    }

    // Typed trades list
    let trades = client
        .market_data
        .get_trades(&TradesQuery {
            size: Some(5),
            ..Default::default()
        })
        .await?;
    println!(
        "\n▶ Recent trades: {} (page {}/{})",
        trades.data.len(),
        trades.pagination.page,
        trades.pagination.last_page
    );
    for trade in &trades.data {
        println!("  · {} — {} @ {:.2} x {}", trade.market_id, trade.outcome, trade.price, trade.size);
    }

    // Sports
    let leagues = client.trading.list_sports_leagues().await?;
    println!("\n▶ Supported leagues: {}", leagues.leagues.len());
    for league in &leagues.leagues {
        println!("  · {} ({})", league.name, league.short_name);
    }

    // ------------------------------------------------------------------
    // 2. Full onboarding — login + create API key in a single call
    // ------------------------------------------------------------------
    let email = env::var("BAYSE_EMAIL").ok();
    let password = env::var("BAYSE_PASSWORD").ok();
    match (email, password) {
        (Some(email), Some(password)) if !email.is_empty() && !password.is_empty() => {
            println!("\n▶ Onboarding with {email} …");
            let (client, key) =
                BayseClient::login_and_create_api_key(&email, &password, "friendly-example")
                    .await?;
            println!("  API key created: {}", key.name);
            println!("  public_key = {}", key.public_key);
            println!("  secret_key = {} (store this — shown only once)", key.secret_key);

            let balances = client.balances().await?;
            println!("\n▶ Wallet balances:");
            for asset in &balances {
                println!(
                    "  · {} — available {}, pending {}",
                    asset.symbol, asset.available_balance, asset.pending_balance
                );
            }
        }
        _ => {
            println!(
                "\n⚠  Skipping onboarding — set BAYSE_EMAIL and BAYSE_PASSWORD"
            );
        }
    }

    println!("\n═══════════════════════════════════════");
    println!("  Friendly API examples complete.");
    println!("═══════════════════════════════════════\n");

    Ok(())
}
