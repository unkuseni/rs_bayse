//! # rs_bayse — Bayse Markets API bindings for Rust
//!
//! `rs_bayse` is a Rust client library for the [Bayse Markets](https://bayse.markets)
//! API. It covers:
//!
//! - **REST API** — Full coverage of all endpoints (system, user, trading, wallet,
//!   market data, and market makers).
//! - **WebSocket API** — Real-time streams for market data, user orders, and
//!   asset prices.
//! - **Authentication** — Public, session-based (token + device ID), read-level
//!   (API key header), and write-level (HMAC-SHA256 signing).
//!
//! ## Quick Start
//!
//! ```no_run
//! use bayse::prelude::*;
//!
//! #[tokio::main]
//! async fn main() -> Result<(), BayseError> {
//!     // Public endpoints (no auth needed)
//!     let sys = SystemManager::new(None, None);
//!     let healthy = sys.health().await?;
//!     println!("API healthy: {healthy}");
//!
//!     // Authenticated endpoints (with API key)
//!     let trading = TradingManager::new(
//!         Some("your_public_key".into()),
//!         Some("your_secret_key".into()),
//!     );
//!     let events = trading.list_events(Some(1), Some(20)).await?;
//!     for event in &events.events {
//!         println!("  · {} [{}]", event.title, event.category);
//!     }
//!
//!     // Get a price quote for a market (public endpoint)
//!     let quote = trading
//!         .get_quote(
//!             "event_id",
//!             "market_id",
//!             &GetQuoteRequest {
//!                 side: "BUY".into(),
//!                 outcome_id: "outcome_id".into(),
//!                 amount: 100.0,
//!                 currency: Some("USD".into()),
//!             },
//!         )
//!         .await?;
//!     println!("Quote price: {}", quote.price);
//!
//!     Ok(())
//! }
//! ```

mod api;
pub mod client;
mod config;
mod errors;
mod market_data;
mod market_maker;
mod models;
mod serde_helpers;
mod system;
mod timed;
mod trading;
mod user;
pub mod util;
mod wallet;
pub mod ws;

/// Convenience re-exports for end users.
pub mod prelude {

    // Re-export top-level types
    pub use crate::api::*;
    pub use crate::client::*;
    pub use crate::config::*;
    pub use crate::errors::*;
    pub use crate::market_data::*;
    pub use crate::market_maker::*;
    pub use crate::models::*;
    pub use crate::serde_helpers::*; // re-export if helpers are added
    pub use crate::system::*;
    pub use crate::timed::*;
    pub use crate::trading::*;
    pub use crate::user::*;
    pub use crate::util::*;
    pub use crate::wallet::*;
    pub use crate::ws::*;

    // Internal re-exports used across modules
    pub(crate) use derive_more::Display;
    pub(crate) use futures::sink::SinkExt;
    pub(crate) use futures::StreamExt;
    pub(crate) use hex::encode as hex_encode;
    pub(crate) use hmac::{Hmac, KeyInit};
    // pub(crate) use log::error;
    pub(crate) use reqwest::header::{CONTENT_TYPE, USER_AGENT};
    pub(crate) use reqwest::Client as ReqwestClient;
    pub(crate) use serde::de::DeserializeOwned;
    pub(crate) use serde::{Deserialize, Serialize};
    pub(crate) use serde_json::Value;
    pub(crate) use sha2::Sha256;
    pub(crate) use std::collections::BTreeMap;
    pub(crate) use thiserror::Error;
    pub(crate) use tokio::net::TcpStream;
    pub(crate) use tokio_tungstenite::WebSocketStream;
    pub(crate) use tokio_tungstenite::{
        connect_async, tungstenite::Message as WsMessage, MaybeTlsStream,
    };
}

pub use prelude::*;
