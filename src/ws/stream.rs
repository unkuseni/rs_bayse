//! High-level WebSocket stream handlers.
//!
//! Provides the [`Stream`] manager with typed subscription methods, an
//! event loop with automatic keepalive, and the [`WebSocketHandler`] trait
//! for processing typed [`WsEvent`] values.

use crate::prelude::*;
use crate::ws::client::WsClient;
use crate::ws::PING_INTERVAL;

use std::time::Instant;
use tokio::sync::mpsc;

/// High-level WebSocket manager.
///
/// Wraps a [`Client`] and provides typed methods for subscribing to each
/// Bayse Markets WebSocket channel.
///
/// # Channel overview
///
/// | Method | Endpoint | Channel | Auth |
/// |--------|----------|---------|------|
/// | [`subscribe_market`](Self::subscribe_market) | `/ws/v1/markets` | `activity`, `prices`, `orderbook`, `user_trades` | None |
/// | [`subscribe_user`](Self::subscribe_user) | `/ws/v1/user` | `orders` | Per-message API key/token |
/// | [`subscribe_realtime`](Self::subscribe_realtime) | `/ws/v1/realtime` | `asset_prices` | None |
pub struct Stream {
    pub client: Client,
}

impl Bayse for Stream {
    fn new(api_key: Option<String>, secret_key: Option<String>) -> Self {
        let config = if api_key.is_some() || secret_key.is_some() {
            Config::default()
                .with_api_key(api_key.unwrap_or_default(), secret_key.unwrap_or_default())
        } else {
            Config::default()
        };
        Self::new_with_config(config)
    }

    fn new_with_config(config: Config) -> Self {
        let client = Client::new(
            config.api_key,
            config.secret_key,
            config.ws_endpoint.to_string(),
            config.session_token,
            config.device_id,
        );
        Self { client }
    }
}

impl Stream {
    // ------------------------------------------------------------------
    // Public subscription methods
    // ------------------------------------------------------------------

    /// Subscribe to **market data** (`/ws/v1/markets`).
    ///
    /// Supports channels: `activity`, `prices`, `orderbook`, `user_trades`.
    /// No authentication is required — the endpoint is public.
    ///
    /// # Parameters
    ///
    /// * `subscription` – A [`WsSubscription`] describing the channel, event,
    ///   market, or symbol filters.
    /// * `handler` – A callback that receives each parsed [`WsEvent`].
    ///
    /// The connection stays open until the handler returns an error or the
    /// stream is closed by the server.
    pub async fn subscribe_market<H>(
        &self,
        subscription: WsSubscription,
        handler: H,
    ) -> Result<(), BayseError>
    where
        H: WebSocketHandler,
    {
        let mut ws_client = WsClient::new(self.client.wss_connect("/ws/v1/markets").await?);
        let sub_msg = serde_json::to_string(&subscription)?;
        ws_client.send_text(&sub_msg).await?;
        Self::event_loop(&mut ws_client, handler, None).await
    }

    /// Subscribe to **market data** with dynamic subscription control.
    ///
    /// Same as [`subscribe_market`](Self::subscribe_market) but accepts a
    /// command channel receiver so you can dynamically subscribe, unsubscribe,
    /// or send pings without reconnecting.
    ///
    /// # Parameters
    ///
    /// * `subscription` – Initial subscription message.
    /// * `handler` – Callback for incoming [`WsEvent`] values.
    /// * `cmd_rx` – Channel receiver for dynamic [`SubscriptionCommand`] values.
    pub async fn subscribe_market_with_commands<H>(
        &self,
        subscription: WsSubscription,
        handler: H,
        cmd_rx: mpsc::UnboundedReceiver<SubscriptionCommand>,
    ) -> Result<(), BayseError>
    where
        H: WebSocketHandler,
    {
        let mut ws_client = WsClient::new(self.client.wss_connect("/ws/v1/markets").await?);
        let sub_msg = serde_json::to_string(&subscription)?;
        ws_client.send_text(&sub_msg).await?;
        Self::event_loop(&mut ws_client, handler, Some(cmd_rx)).await
    }

    /// Subscribe to **user order updates** (`/ws/v1/user`).
    ///
    /// Each message sent on this endpoint must include `auth` credentials.
    /// The initial subscription can carry the auth field; subsequent commands
    /// from the channel may also include it.
    ///
    /// Supports the `orders` channel.
    ///
    /// # Parameters
    ///
    /// * `subscription` – A [`WsSubscription`] with `.with_auth(...)` set.
    /// * `handler` – A callback that receives each parsed [`WsEvent`].
    pub async fn subscribe_user<H>(
        &self,
        subscription: WsSubscription,
        handler: H,
    ) -> Result<(), BayseError>
    where
        H: WebSocketHandler,
    {
        let mut ws_client = WsClient::new(self.client.wss_connect("/ws/v1/user").await?);
        let sub_msg = serde_json::to_string(&subscription)?;
        ws_client.send_text(&sub_msg).await?;
        Self::event_loop(&mut ws_client, handler, None).await
    }

    /// Subscribe to **user order updates** with dynamic subscription control.
    ///
    /// Same as [`subscribe_user`](Self::subscribe_user) but accepts a
    /// command channel for dynamic subscribe/unsubscribe/ping.
    pub async fn subscribe_user_with_commands<H>(
        &self,
        subscription: WsSubscription,
        handler: H,
        cmd_rx: mpsc::UnboundedReceiver<SubscriptionCommand>,
    ) -> Result<(), BayseError>
    where
        H: WebSocketHandler,
    {
        let mut ws_client = WsClient::new(self.client.wss_connect("/ws/v1/user").await?);
        let sub_msg = serde_json::to_string(&subscription)?;
        ws_client.send_text(&sub_msg).await?;
        Self::event_loop(&mut ws_client, handler, Some(cmd_rx)).await
    }

    /// Subscribe to **real-time asset prices** (`/ws/v1/realtime`).
    ///
    /// Supports the `asset_prices` channel. No authentication is required.
    ///
    /// # Parameters
    ///
    /// * `subscription` – A [`WsSubscription`] with symbols set.
    /// * `handler` – A callback that receives each parsed [`WsEvent`].
    pub async fn subscribe_realtime<H>(
        &self,
        subscription: WsSubscription,
        handler: H,
    ) -> Result<(), BayseError>
    where
        H: WebSocketHandler,
    {
        let mut ws_client = WsClient::new(self.client.wss_connect("/ws/v1/realtime").await?);
        let sub_msg = serde_json::to_string(&subscription)?;
        ws_client.send_text(&sub_msg).await?;
        Self::event_loop(&mut ws_client, handler, None).await
    }

    // ------------------------------------------------------------------
    // Event loop
    // ------------------------------------------------------------------

    /// Core event loop that reads messages, dispatches to the handler,
    /// sends periodic pings, and processes subscription commands.
    ///
    /// The server may batch multiple JSON messages into a single WebSocket
    /// frame separated by newlines. This method splits on `\n` before
    /// parsing each line.
    ///
    /// # Type Parameters
    ///
    /// * `H` – A [`WebSocketHandler`] that processes parsed [`WsEvent`] values.
    async fn event_loop<H>(
        client: &mut WsClient,
        mut handler: H,
        mut cmd_rx: Option<mpsc::UnboundedReceiver<SubscriptionCommand>>,
    ) -> Result<(), BayseError>
    where
        H: WebSocketHandler,
    {
        let mut last_ping = Instant::now();

        loop {
            // ----- Check for subscription commands -----
            if let Some(ref mut rx) = cmd_rx {
                loop {
                    match rx.try_recv() {
                        Ok(SubscriptionCommand::Subscribe(sub)) => {
                            let msg = serde_json::to_string(&sub)?;
                            client.send_text(&msg).await?;
                        }
                        Ok(SubscriptionCommand::Unsubscribe(sub)) => {
                            let msg = serde_json::to_string(&sub)?;
                            client.send_text(&msg).await?;
                        }
                        Ok(SubscriptionCommand::SendText(text)) => {
                            client.send_text(&text).await?;
                        }
                        Err(mpsc::error::TryRecvError::Empty) => break,
                        Err(mpsc::error::TryRecvError::Disconnected) => {
                            cmd_rx = None;
                            break;
                        }
                    }
                }
            }

            // ----- Ping keepalive -----
            if last_ping.elapsed() > PING_INTERVAL {
                let ping = WsSubscription::ping();
                let msg = serde_json::to_string(&ping)?;
                client.send_text(&msg).await?;
                last_ping = Instant::now();
            }

            // ----- Read next message -----
            let msg = tokio::time::timeout(PING_INTERVAL, client.stream().next()).await;

            match msg {
                Ok(Some(Ok(WsMessage::Text(text)))) => {
                    // Server may batch lines separated by \n
                    for line in text.lines() {
                        let trimmed = line.trim();
                        if trimmed.is_empty() {
                            continue;
                        }
                        handler.handle_msg(trimmed)?;
                    }
                }
                Ok(Some(Ok(WsMessage::Ping(data)))) => {
                    let _ = client.stream().send(WsMessage::Pong(data)).await;
                }
                Ok(Some(Ok(WsMessage::Pong(_)))) => {
                    // Protocol-level pong received; no action needed
                }
                // Binary, Close, Frame — no action needed
                Ok(Some(Ok(_))) => {}
                Ok(Some(Err(e))) => {
                    return Err(BayseError::Base(format!("WebSocket stream error: {e}")));
                }
                Ok(None) => {
                    return Err(BayseError::Base("WebSocket stream closed by server".into()));
                }
                Err(_) => {
                    // Timeout on the read — loop back to ping/cmd check
                }
            }
        }
    }

    /// Gracefully disconnect a WebSocket client.
    pub async fn disconnect(&self, client: &mut WsClient) -> Result<(), BayseError> {
        client.disconnect().await
    }
}

// ---------------------------------------------------------------------------
// WebSocketHandler trait
// ---------------------------------------------------------------------------

/// Trait for processing raw JSON lines from a WebSocket stream.
///
/// The trait handles JSON deserialisation internally via
/// [`WsEvent::from_json`], so implementors only work with typed
/// [`WsEvent`] values. Use the blanket impl to pass a closure:
/// `\|event: WsEvent\| { ... Ok(()) }`.
pub trait WebSocketHandler {
    /// Process a single raw JSON line received from the server.
    ///
    /// The default implementation calls [`WsEvent::from_json`] to parse
    /// the line and then calls [`on_event`](Self::on_event).  Override
    /// this method if you need custom parsing or want to handle malformed
    /// messages gracefully.
    ///
    /// # Errors
    ///
    /// Return `Err` to break out of the event loop.
    fn handle_msg(&mut self, msg: &str) -> Result<(), BayseError> {
        let event = WsEvent::from_json(msg)?;
        self.on_event(event)
    }

    /// Process a parsed [`WsEvent`].
    ///
    /// # Errors
    ///
    /// Return `Err` to break out of the event loop.
    fn on_event(&mut self, event: WsEvent) -> Result<(), BayseError>;
}

impl<F> WebSocketHandler for F
where
    F: FnMut(WsEvent) -> Result<(), BayseError>,
{
    fn on_event(&mut self, event: WsEvent) -> Result<(), BayseError> {
        self(event)
    }
}

// ---------------------------------------------------------------------------
// SubscriptionCommand
// ---------------------------------------------------------------------------

/// A command sent through the dynamic subscription channel to control an
/// active WebSocket connection without reconnecting.
#[derive(Debug, Clone)]
pub enum SubscriptionCommand {
    /// Subscribe to a new channel or update filters.
    Subscribe(WsSubscription),
    /// Unsubscribe from a room.
    Unsubscribe(WsSubscription),
    /// Send an arbitrary text message (e.g. a raw ping).
    SendText(String),
}
