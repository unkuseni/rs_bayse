//! # WebSocket API Examples for Bayse Markets
//!
//! Demonstrates how to subscribe to real-time data streams using the typed
//! [`WsEvent`] API.
//!
//! ## Running
//!
//! ```bash
//! # Market data stream (price updates)
//! cargo run --example websocket market <EVENT_ID> <DURATION>
//!
//! # Realtime asset prices
//! cargo run --example websocket realtime <DURATION>
//!
//! # User order updates (requires BAYSE_API_KEY)
//! BAYSE_API_KEY="pk_live_..." cargo run --example websocket user <DURATION>
//! ```

use bayse::prelude::*;
use bayse::ws::WsEvent;
use std::env;
use tokio::time::Duration;

/// Available demo modes.
enum Mode {
    Market,
    User,
    Realtime,
}

impl Mode {
    fn from_str(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "market" | "markets" => Some(Self::Market),
            "user" | "orders" => Some(Self::User),
            "realtime" | "prices" | "fx" | "crypto" => Some(Self::Realtime),
            _ => None,
        }
    }
}

async fn run_with_timeout(duration_secs: u64, fut: impl std::future::Future<Output = ()>) {
    if duration_secs > 0 {
        tokio::select! {
            _ = fut => {}
            _ = tokio::time::sleep(Duration::from_secs(duration_secs)) => {
                println!("⏱  Time limit reached ({duration_secs}s).");
            }
        }
    } else {
        fut.await;
    }
}

// ---------------------------------------------------------------------------
// Demo 1 — Market data stream (price updates for an event)
// ---------------------------------------------------------------------------
async fn demo_market(event_id: &str, duration: u64) {
    println!("▶ Subscribing to market data for event {event_id} …");
    let ws: Stream = Bayse::new(None, None);
    let event_id = event_id.to_owned();

    run_with_timeout(duration, async move {
        let sub = WsSubscription::new("subscribe", "prices").with_event_id(&event_id);
        let _ = ws
            .subscribe_market(sub, |event: WsEvent| {
                match event {
                    WsEvent::PriceUpdate(e) => {
                        println!("  Price update for {}:", e.data.title);
                        for market in &e.data.markets {
                            println!(
                                "    {} — YES: {:.4}, NO: {:.4}",
                                market.question,
                                market.prices.get("YES").unwrap_or(&0.0),
                                market.prices.get("NO").unwrap_or(&0.0),
                            );
                        }
                    }
                    WsEvent::Connected(_) => println!("  Connected!"),
                    WsEvent::Pong(_) => {}
                    other => println!("  Other event: {other:?}"),
                }
                Ok(())
            })
            .await;
    })
    .await;
}

// ---------------------------------------------------------------------------
// Demo 2 — User order stream (requires API key)
// ---------------------------------------------------------------------------
async fn demo_user(api_key: &str, market_ids: &[String], duration: u64) {
    println!("▶ Subscribing to user order updates …");
    let ws: Stream = Bayse::new(Some(api_key.into()), None);
    let auth = WsAuth::with_api_key(api_key);
    let market_ids = market_ids.to_vec();

    run_with_timeout(duration, async move {
        let sub = WsSubscription::new("subscribe", "orders")
            .with_market_ids(market_ids)
            .with_auth(auth);
        let _ = ws
            .subscribe_user(sub, |event: WsEvent| {
                match event {
                    WsEvent::OrderUpdated(e) => {
                        println!(
                            "  Order {} — side={}, filled={}, status={}",
                            e.data.order_id,
                            e.data.order.side,
                            e.data.order.filled_quantity,
                            e.data.order.status,
                        );
                    }
                    WsEvent::Connected(_) => println!("  Connected!"),
                    WsEvent::Pong(_) => {}
                    other => println!("  Other event: {other:?}"),
                }
                Ok(())
            })
            .await;
    })
    .await;
}

// ---------------------------------------------------------------------------
// Demo 3 — Realtime asset prices
// ---------------------------------------------------------------------------
async fn demo_realtime(duration: u64) {
    println!("▶ Subscribing to realtime asset prices …");
    let ws: Stream = Bayse::new(None, None);

    run_with_timeout(duration, async move {
        let sub = WsSubscription::new("subscribe", "asset_prices")
            .with_symbols(vec!["BTCUSDT".into(), "ETHUSDT".into()]);
        let _ = ws
            .subscribe_realtime(sub, |event: WsEvent| {
                match event {
                    WsEvent::AssetPrice(e) => {
                        println!("  {}: ${:.2}", e.data.symbol, e.data.price);
                    }
                    WsEvent::Connected(_) => println!("  Connected!"),
                    WsEvent::Pong(_) => {}
                    other => println!("  Other event: {other:?}"),
                }
                Ok(())
            })
            .await;
    })
    .await;
}

fn print_usage() {
    eprintln!(
        r#"Usage: cargo run --example websocket <MODE> [ARGS] [DURATION]

Modes:
  market <EVENT_ID> [DURATION]
      – Subscribe to price updates for an event
  user <MARKET_IDS...> [DURATION]
      – Subscribe to user order updates (needs BAYSE_API_KEY)
  realtime [DURATION]
      – Subscribe to realtime asset prices (BTC, ETH)

Duration in seconds (default: 30, 0 = run forever)

Environment:
  BAYSE_API_KEY       API key (required for user mode)
"#,
    );
}

#[tokio::main]
async fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() < 2 {
        print_usage();
        std::process::exit(1);
    }

    let mode = match Mode::from_str(&args[1]) {
        Some(m) => m,
        None => {
            eprintln!("Unknown mode: {}", args[1]);
            print_usage();
            std::process::exit(1);
        }
    };

    match mode {
        Mode::Market => {
            let event_id = args.get(2).map(|s| s.as_str()).unwrap_or("example_event");
            let duration: u64 = args.get(3).and_then(|s| s.parse().ok()).unwrap_or(30);
            demo_market(event_id, duration).await;
        }
        Mode::User => {
            let api_key = env::var("BAYSE_API_KEY").unwrap_or_default();
            if api_key.is_empty() {
                eprintln!("BAYSE_API_KEY is required for user mode");
                std::process::exit(1);
            }
            // All remaining args (after "user") are market IDs; last may be duration
            let market_ids: Vec<String> = args[2..]
                .iter()
                .filter(|s| s.parse::<u64>().is_err()) // skip duration
                .cloned()
                .collect();
            let duration: u64 = args.last().and_then(|s| s.parse().ok()).unwrap_or(30);
            demo_user(&api_key, &market_ids, duration).await;
        }
        Mode::Realtime => {
            let duration: u64 = args.get(2).and_then(|s| s.parse().ok()).unwrap_or(30);
            demo_realtime(duration).await;
        }
    }
}
