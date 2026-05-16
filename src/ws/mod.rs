//! WebSocket module for real-time data feeds.
//!
//! Bayse Markets exposes three WebSocket endpoints:
//!
//! | Endpoint | Auth | Description |
//! |----------|------|-------------|
//! | `/ws/v1/markets` | None | Market activity, price updates, order book snapshots |
//! | `/ws/v1/user` | Per-message | Fill updates for your orders |
//! | `/ws/v1/realtime` | None | Live crypto and FX asset prices |
//!
//! # Architecture
//!
//! The module provides three layers:
//!
//! 1. **`WsClient`** — Low-level connection wrapper (connect, send, read, disconnect).
//! 2. **`Stream`** — High-level manager with typed subscription methods.
//! 3. **`WsEvent`** — Typed enum covering all server event types.
//!
//! # Example
//!
//! ```no_run
//! use bayse::prelude::*;
//! use bayse::ws::WsEvent;
//!
//! #[tokio::main]
//! async fn main() -> Result<(), BayseError> {
//!     let stream = Stream::new(None, None);
//!
//!     stream.subscribe_market(
//!         WsSubscription::new("subscribe", "prices")
//!             .with_event_id("evt_123"),
//!         |event: WsEvent| {
//!             println!("Received: {event:?}");
//!             Ok(())
//!         },
//!     ).await
//! }
//! ```

pub mod client;
pub mod stream;

pub use stream::*;

use serde::{Deserialize, Serialize};
use tokio::time::Duration;

use crate::prelude::*;
use tokio::sync::mpsc;

/// Interval between application-level ping frames sent to keep the
/// connection alive.  The server-side timeout is ~60 s.
pub(crate) const PING_INTERVAL: Duration = Duration::from_secs(30);

/// Helper to send an item through an unbounded channel, mapping the error
/// to [`BayseError::ChannelSendError`].
#[allow(dead_code)]
pub(crate) fn send_or_err<T>(sender: &mpsc::UnboundedSender<T>, item: T) -> Result<(), BayseError> {
    sender.send(item).map_err(|e| BayseError::ChannelSendError {
        underlying: e.to_string(),
    })
}

// ---------------------------------------------------------------------------
// Typed WebSocket event types
// ---------------------------------------------------------------------------

/// A WebSocket message received from the Bayse Markets server.
///
/// Each variant wraps a typed struct.  Use [`WsEvent::from_json`] to
/// parse a raw JSON line by dispatching on the `"type"` field.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum WsEvent {
    /// Initial connection confirmation.
    Connected(ConnectedEvent),
    /// A buy or sell order was filled (activity / user_trades channels).
    TradeOrder(TradeOrderEvent),
    /// Market price changed (prices channel).
    PriceUpdate(PriceUpdateEvent),
    /// Order book snapshot was updated (orderbook channel).
    OrderbookUpdate(OrderbookUpdateEvent),
    /// Your own order was partially or fully filled (user orders channel).
    OrderUpdated(OrderUpdatedEvent),
    /// Live asset price tick (realtime channel).
    AssetPrice(AssetPriceEvent),
    /// Subscription confirmed.
    Subscribed(SubscribedEvent),
    /// Unsubscription confirmed.
    Unsubscribed(UnsubscribedEvent),
    /// Pong response to a client ping.
    Pong(PongEvent),
    /// Server-side error.
    Error(ErrorEvent),
}

impl WsEvent {
    /// Parse a single JSON line into a [`WsEvent`] by dispatching on
    /// the `"type"` field.
    ///
    /// # Errors
    ///
    /// Returns [`BayseError::JsonError`] if the JSON is malformed or the
    /// `"type"` field does not match a known variant.
    pub fn from_json(s: &str) -> Result<Self, BayseError> {
        let v: Value = serde_json::from_str(s)?;
        let type_str = v["type"]
            .as_str()
            .ok_or_else(|| BayseError::Base("missing 'type' field".into()))?;
        match type_str {
            "connected" => Ok(WsEvent::Connected(serde_json::from_value(v)?)),
            "buy_order" | "sell_order" => Ok(WsEvent::TradeOrder(serde_json::from_value(v)?)),
            "price_update" => Ok(WsEvent::PriceUpdate(serde_json::from_value(v)?)),
            "orderbook_update" => Ok(WsEvent::OrderbookUpdate(serde_json::from_value(v)?)),
            "order_updated" => Ok(WsEvent::OrderUpdated(serde_json::from_value(v)?)),
            "asset_price" => Ok(WsEvent::AssetPrice(serde_json::from_value(v)?)),
            "subscribed" => Ok(WsEvent::Subscribed(serde_json::from_value(v)?)),
            "unsubscribed" => Ok(WsEvent::Unsubscribed(serde_json::from_value(v)?)),
            "pong" => Ok(WsEvent::Pong(serde_json::from_value(v)?)),
            "error" => Ok(WsEvent::Error(serde_json::from_value(v)?)),
            other => {
                return Err(BayseError::Base(format!("unknown WsEvent type: {other}")));
            }
        }
    }
}

/// Initial connection confirmation.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ConnectedEvent {
    /// Always `"connected"`.
    #[serde(rename = "type")]
    pub _type: String,
    /// Connection status.
    pub status: String,
    /// Assigned client UUID.
    pub client_id: String,
    /// Human-readable connection message.
    pub message: String,
    /// Server timestamp (Unix ms).
    pub timestamp: u64,
}

/// A buy or sell order fill event.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TradeOrderEvent {
    #[serde(rename = "type")]
    pub _type: String,
    pub data: TradeOrderData,
    pub timestamp: u64,
}

/// Market price update event.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PriceUpdateEvent {
    #[serde(rename = "type")]
    pub _type: String,
    pub data: PriceUpdateData,
    pub timestamp: u64,
}

/// Order book snapshot update event.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OrderbookUpdateEvent {
    #[serde(rename = "type")]
    pub _type: String,
    pub data: OrderbookUpdateData,
    pub timestamp: u64,
}

/// User order fill update event.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OrderUpdatedEvent {
    #[serde(rename = "type")]
    pub _type: String,
    pub data: OrderUpdatedData,
    pub timestamp: u64,
}

/// Asset price tick event.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AssetPriceEvent {
    #[serde(rename = "type")]
    pub _type: String,
    pub data: AssetPriceData,
    pub timestamp: u64,
}

/// Subscription confirmed event.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SubscribedEvent {
    #[serde(rename = "type")]
    pub _type: String,
    pub room: String,
    #[serde(default)]
    pub message: Option<String>,
    #[serde(default)]
    pub timestamp: Option<u64>,
}

/// Unsubscription confirmed event.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UnsubscribedEvent {
    #[serde(rename = "type")]
    pub _type: String,
    pub room: String,
    #[serde(default)]
    pub message: Option<String>,
    #[serde(default)]
    pub timestamp: Option<u64>,
}

/// Pong response to a client ping.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PongEvent {
    #[serde(rename = "type")]
    pub _type: String,
    pub timestamp: u64,
}

/// Server-side error event.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ErrorEvent {
    #[serde(rename = "type")]
    pub _type: String,
    pub data: WsErrorData,
    pub timestamp: u64,
}

/// Data payload for trade order events (`buy_order` / `sell_order`).
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TradeOrderData {
    pub user: TradeUserInfo,
    pub order: TradeOrderInfo,
    pub event: TradeEventInfo,
    pub market: TradeMarketInfo,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TradeUserInfo {
    pub id: String,
    pub tag: Option<String>,
    pub image_url: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TradeOrderInfo {
    pub id: String,
    pub amount: f64,
    pub quantity: f64,
    pub price: f64,
    pub status: String,
    #[serde(rename = "type")]
    pub side: String,
    pub outcome: Option<String>,
    pub outcome_label: Option<String>,
    pub currency: String,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TradeEventInfo {
    pub id: String,
    pub slug: String,
    pub title: String,
    #[serde(rename = "type")]
    pub event_type: String,
    pub created_at: String,
    pub image_url: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TradeMarketInfo {
    pub id: String,
    pub title: String,
    pub image_url: Option<String>,
}

/// Data payload for price update events.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PriceUpdateData {
    pub id: String,
    pub slug: String,
    pub title: String,
    pub status: String,
    #[serde(rename = "type")]
    pub event_type: String,
    pub markets: Vec<PriceUpdateMarket>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PriceUpdateMarket {
    pub id: String,
    pub question: String,
    pub outcomes: Vec<String>,
    pub engine: String,
    pub prices: std::collections::HashMap<String, f64>,
}

/// Data payload for order book update events.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OrderbookUpdateData {
    pub orderbook: OrderbookSnapshot,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OrderbookSnapshot {
    pub market_id: String,
    pub outcome_id: Option<String>,
    pub timestamp: String,
    pub bids: Vec<OrderbookLevel>,
    pub asks: Vec<OrderbookLevel>,
    pub last_traded_price: Option<f64>,
    pub last_traded_side: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OrderbookLevel {
    pub price: f64,
    pub quantity: f64,
    pub total: f64,
}

/// Data payload for user order update events.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OrderUpdatedData {
    pub order_id: String,
    pub event_id: String,
    pub market_id: String,
    pub order: UserOrderInfo,
    pub timestamp: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UserOrderInfo {
    pub id: String,
    pub user_id: String,
    pub market_id: String,
    pub outcome_id: Option<String>,
    pub outcome_label: Option<String>,
    pub side: String,
    pub price: f64,
    pub quantity: f64,
    pub filled_quantity: f64,
    pub remaining_quantity: f64,
    pub avg_fill_price: Option<f64>,
    pub status: String,
    pub time_in_force: Option<String>,
    pub created_at: u64,
    pub updated_at: u64,
}

/// Data payload for asset price events.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AssetPriceData {
    pub symbol: String,
    pub price: f64,
    pub timestamp: u64,
}

/// Error payload in a server error message.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WsErrorData {
    pub message: String,
}
