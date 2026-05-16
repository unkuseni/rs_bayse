//! WebSocket module for real-time data feeds.
//!
//! Bayse Markets exposes three WebSocket endpoints:
//! - `/ws/v1/markets` – Market data (public, no auth)
//! - `/ws/v1/user` – User orders (per-message auth)
//! - `/ws/v1/realtime` – Asset prices (public, no auth)

pub mod client;
pub mod stream;

pub use stream::*;

use tokio::time::Duration;

use crate::prelude::*;
use tokio::sync::mpsc;

/// Interval at which the WebSocket event loop sends a ping to keep the connection alive.
///
/// If no data is received within this window a WebSocket Ping frame is
/// sent. If the peer does not respond with a Pong the connection is
/// considered stale and will be dropped by the runtime.
#[allow(dead_code)]
pub(crate) const PING_INTERVAL: Duration = Duration::from_secs(30);

/// Helper to send an item through an unbounded channel, mapping the error to `BayseError`.
///
/// This is a convenience wrapper around [`mpsc::UnboundedSender::send`]
/// that converts the channel's [`SendError`](mpsc::error::SendError) into
/// a [`BayseError::ChannelSendError`] so callers can use the `?` operator
/// without writing the `map_err` boilerplate every time.
///
/// # Parameters
///
/// * `sender` – The unbounded sender to push the item into.
/// * `item` – The value to send through the channel.
///
/// # Errors
///
/// Returns [`BayseError::ChannelSendError`] if the receiving half has
/// been dropped (the channel is closed).
#[allow(dead_code)]
pub(crate) fn send_or_err<T>(sender: &mpsc::UnboundedSender<T>, item: T) -> Result<(), BayseError> {
    sender.send(item).map_err(|e| BayseError::ChannelSendError {
        underlying: e.to_string(),
    })
}
