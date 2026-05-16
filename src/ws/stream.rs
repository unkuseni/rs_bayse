//! High-level WebSocket stream handlers.
//!
//! Provides subscription management and an event loop that processes incoming
//! messages through a user-provided callback.

use crate::prelude::*;
use crate::ws::client::WsClient;

/// High-level WebSocket manager.
///
/// Wraps a `Client` and provides methods to subscribe to various real-time
/// channels.
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
            // For WebSocket, we use the WS host; replace the scheme
            config.ws_endpoint.to_string(),
            config.session_token,
            config.device_id,
        );
        Self { client }
    }
}

impl Stream {
    /// Subscribe to a market data channel.
    ///
    /// Opens a WebSocket connection to `/ws/v1/markets` and sends a subscription
    /// message. Incoming messages are delivered to the `handler` callback.
    ///
    /// The server may batch multiple JSON messages into a single WebSocket frame
    /// separated by newlines. This method splits on `\n` before parsing.
    pub async fn subscribe_market<H, E>(
        &self,
        subscription: WsSubscription,
        handler: H,
    ) -> Result<(), BayseError>
    where
        H: Fn(Value) -> Result<(), E> + Send + 'static,
        E: std::error::Error + Send + 'static,
    {
        let mut ws_client = WsClient::new(self.client.wss_connect("/ws/v1/markets").await?);

        // Send subscription
        let sub_msg = serde_json::to_string(&subscription)?;
        ws_client
            .stream()
            .send(WsMessage::Text(sub_msg.into()))
            .await
            .map_err(|e| BayseError::Base(format!("Send error: {e}")))?;

        // Event loop
        loop {
            match ws_client.stream().next().await {
                Some(Ok(msg)) => {
                    if let WsMessage::Text(text) = msg {
                        // The server may batch multiple JSON objects separated by \n
                        for line in text.lines() {
                            if line.trim().is_empty() {
                                continue;
                            }
                            match serde_json::from_str::<Value>(line) {
                                Ok(val) => {
                                    if let Err(e) = handler(val) {
                                        log::error!("Handler error: {e}");
                                    }
                                }
                                Err(e) => {
                                    log::error!("Failed to parse message: {e}");
                                }
                            }
                        }
                    }
                }
                Some(Err(e)) => {
                    log::error!("WebSocket error: {e}");
                    break;
                }
                None => break,
            }
        }

        Ok(())
    }

    /// Subscribe to user order updates.
    ///
    /// Opens a WebSocket connection to `/ws/v1/user`. Each message must be
    /// authenticated with an API key. The `auth_message` closure should return
    /// the JSON string to send for authentication.
    pub async fn subscribe_user<H, E>(
        &self,
        auth_message: String,
        subscription: WsSubscription,
        handler: H,
    ) -> Result<(), BayseError>
    where
        H: Fn(Value) -> Result<(), E> + Send + 'static,
        E: std::error::Error + Send + 'static,
    {
        let mut ws_client = WsClient::new(self.client.wss_connect("/ws/v1/user").await?);

        // Send authentication
        ws_client
            .stream()
            .send(WsMessage::Text(auth_message.into()))
            .await
            .map_err(|e| BayseError::Base(format!("Send error: {e}")))?;

        // Send subscription
        let sub_msg = serde_json::to_string(&subscription)?;
        ws_client
            .stream()
            .send(WsMessage::Text(sub_msg.into()))
            .await
            .map_err(|e| BayseError::Base(format!("Send error: {e}")))?;

        // Event loop
        loop {
            match ws_client.stream().next().await {
                Some(Ok(msg)) => {
                    if let WsMessage::Text(text) = msg {
                        for line in text.lines() {
                            if line.trim().is_empty() {
                                continue;
                            }
                            match serde_json::from_str::<Value>(line) {
                                Ok(val) => {
                                    if let Err(e) = handler(val) {
                                        log::error!("Handler error: {e}");
                                    }
                                }
                                Err(e) => {
                                    log::error!("Failed to parse message: {e}");
                                }
                            }
                        }
                    }
                }
                Some(Err(e)) => {
                    log::error!("WebSocket error: {e}");
                    break;
                }
                None => break,
            }
        }

        Ok(())
    }

    /// Subscribe to real-time asset prices (crypto and FX).
    ///
    /// Opens a WebSocket connection to `/ws/v1/realtime`.
    pub async fn subscribe_realtime<H, E>(
        &self,
        subscription: WsSubscription,
        handler: H,
    ) -> Result<(), BayseError>
    where
        H: Fn(Value) -> Result<(), E> + Send + 'static,
        E: std::error::Error + Send + 'static,
    {
        let mut ws_client = WsClient::new(self.client.wss_connect("/ws/v1/realtime").await?);

        // Send subscription
        let sub_msg = serde_json::to_string(&subscription)?;
        ws_client
            .stream()
            .send(WsMessage::Text(sub_msg.into()))
            .await
            .map_err(|e| BayseError::Base(format!("Send error: {e}")))?;

        // Event loop
        loop {
            match ws_client.stream().next().await {
                Some(Ok(msg)) => {
                    if let WsMessage::Text(text) = msg {
                        for line in text.lines() {
                            if line.trim().is_empty() {
                                continue;
                            }
                            match serde_json::from_str::<Value>(line) {
                                Ok(val) => {
                                    if let Err(e) = handler(val) {
                                        log::error!("Handler error: {e}");
                                    }
                                }
                                Err(e) => {
                                    log::error!("Failed to parse message: {e}");
                                }
                            }
                        }
                    }
                }
                Some(Err(e)) => {
                    log::error!("WebSocket error: {e}");
                    break;
                }
                None => break,
            }
        }

        Ok(())
    }

    /// Disconnect the WebSocket stream.
    pub async fn disconnect(&self) -> Result<(), BayseError> {
        // The `ws_subscribe` methods own the connection, so this is a no-op
        // for the current design. Users should break out of the event loop
        // to disconnect.
        Ok(())
    }
}
