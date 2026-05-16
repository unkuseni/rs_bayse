//! # WebSocket API Examples for Bayse Markets
//!
//! Demonstrates how to subscribe to real-time data streams.
//!
//! ## Running
//!
//! ```bash
//! # Market data stream
//! cargo run --example websocket market
//!
//! # Asset prices stream
//! cargo run --example websocket realtime
//! ```

use bayse::prelude::*;
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
// Demo 1 – Market data stream
// ---------------------------------------------------------------------------
async fn demo_market(event_id: &str, duration: u64) {
    println!("▶ Subscribing to market data for event {event_id} …");
    let ws: Stream = Bayse::new(None, None);
    let event_id_owned = event_id.to_owned();

    run_with_timeout(duration, async move {
        let sub = WsSubscription::new("subscribe", "prices").with_event_id(event_id_owned);
        let _ = ws
            .subscribe_market(sub, |msg| {
                println!("  Market data: {msg:#?}");
                Ok::<_, std::io::Error>(())
            })
            .await;
    })
    .await;
}

// ---------------------------------------------------------------------------
// Demo 2 – User order stream (requires API key)
// ---------------------------------------------------------------------------
async fn demo_user(api_key: &str, _api_secret: &str, duration: u64) {
    println!("▶ Subscribing to user order updates …");
    let ws: Stream = Bayse::new(Some(api_key.into()), None);

    run_with_timeout(duration, async move {
        // Per-message authentication payload — structure depends on Bayse's spec
        let auth_msg = serde_json::json!({
            "type": "auth",
            "apiKey": api_key,
        })
        .to_string();

        let sub = WsSubscription::new("subscribe", "orders");
        let _ = ws
            .subscribe_user(auth_msg, sub, |msg| {
                println!("  User order update: {msg:#?}");
                Ok::<_, std::io::Error>(())
            })
            .await;
    })
    .await;
}

// ---------------------------------------------------------------------------
// Demo 3 – Realtime asset prices
// ---------------------------------------------------------------------------
async fn demo_realtime(duration: u64) {
    println!("▶ Subscribing to realtime asset prices …");
    let ws: Stream = Bayse::new(None, None);

    run_with_timeout(duration, async move {
        let sub = WsSubscription::new("subscribe", "prices");
        let _ = ws
            .subscribe_realtime(sub, |msg| {
                println!("  Price update: {msg:#?}");
                Ok::<_, std::io::Error>(())
            })
            .await;
    })
    .await;
}

fn print_usage() {
    eprintln!(
        r#"Usage: cargo run --example websocket <MODE> [EVENT_ID] [DURATION]

Modes:
  market    – Subscribe to market data feed (default event: "example_event")
  user      – Subscribe to user order updates (needs BAYSE_API_KEY)
  realtime  – Subscribe to realtime asset prices

Event ID (for market mode, default: "example_event")
Duration in seconds (default: 30, 0 = run forever)

Environment:
  BAYSE_API_KEY       API key (required for user mode)
  BAYSE_API_SECRET    API secret (required for user mode)
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

    let event_id = args.get(2).map(|s| s.as_str()).unwrap_or("example_event");
    let duration: u64 = args.get(3).and_then(|s| s.parse().ok()).unwrap_or(30);

    match mode {
        Mode::Market => demo_market(event_id, duration).await,
        Mode::User => {
            let api_key = env::var("BAYSE_API_KEY").unwrap_or_default();
            let api_secret = env::var("BAYSE_API_SECRET").unwrap_or_default();
            if api_key.is_empty() {
                eprintln!("BAYSE_API_KEY is required for user mode");
                std::process::exit(1);
            }
            demo_user(&api_key, &api_secret, duration).await;
        }
        Mode::Realtime => demo_realtime(duration).await,
    }
}
