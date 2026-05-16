//! Low-level WebSocket connection wrapper.
//!
//! Provides connect, send, read, and disconnect with proper Close-frame
//! semantics and automatic pong replies.

use crate::prelude::*;

/// Manages a single WebSocket connection lifecycle.
///
/// Wraps a connected `WebSocketStream<MaybeTlsStream<TcpStream>>` and
/// provides methods to send text messages, read incoming frames (with
/// automatic pong replies), and gracefully disconnect.
///
/// # Example
///
/// ```no_run
/// use bayse::prelude::*;
///
/// async fn example(client: &Client) -> Result<(), BayseError> {
///     let stream = client.wss_connect("/ws/v1/markets").await?;
///     let mut ws = WsClient::new(stream);
///
///     // Send a ping
///     ws.send_text(r#"{"type":"ping"}"#).await?;
///
///     // Read the next message
///     if let Some(msg) = ws.read_text().await {
///         println!("Received: {msg}");
///     }
///
///     ws.disconnect().await?;
///     Ok(())
/// }
/// ```
pub struct WsClient {
    /// The underlying WebSocket stream used for reading and writing.
    stream: WebSocketStream<MaybeTlsStream<TcpStream>>,
}

impl WsClient {
    /// Wrap a connected stream (typically from [`Client::wss_connect`]).
    pub fn new(stream: WebSocketStream<MaybeTlsStream<TcpStream>>) -> Self {
        Self { stream }
    }

    /// Get a mutable reference to the underlying stream for custom usage.
    pub fn stream(&mut self) -> &mut WebSocketStream<MaybeTlsStream<TcpStream>> {
        &mut self.stream
    }

    /// Send a text message over the WebSocket connection.
    ///
    /// Serialises the provided string as a WebSocket text frame.
    /// Returns an error if the connection has been closed or if the
    /// underlying stream encounters a write error.
    pub async fn send_text(&mut self, text: &str) -> Result<(), BayseError> {
        self.stream
            .send(WsMessage::Text(text.into()))
            .await
            .map_err(|e| BayseError::Base(format!("WebSocket send error: {e}")))
    }

    /// Read the next text message from the WebSocket connection.
    ///
    /// Automatically responds to server ping frames with pong replies.
    /// Binary frames and other non-text message types are silently skipped.
    /// Returns `None` when the stream is exhausted (connection closed).
    pub async fn read_text(&mut self) -> Option<String> {
        loop {
            match self.stream.next().await? {
                Ok(WsMessage::Text(text)) => return Some(text.to_string()),
                // Respond to server ping frames automatically
                Ok(WsMessage::Ping(data)) => {
                    let _ = self.stream.send(WsMessage::Pong(data)).await;
                }
                // Pong replies are handled by the library; no action needed
                Ok(WsMessage::Pong(_)) => {}
                // Ignore binary and other frames
                Ok(_) => {}
                Err(e) => {
                    log::error!("WebSocket read error: {e}");
                    return None;
                }
            }
        }
    }

    /// Send a WebSocket Close frame and consume the connection.
    pub async fn disconnect(&mut self) -> Result<(), BayseError> {
        self.stream
            .close(None)
            .await
            .map_err(|e| BayseError::Base(format!("Error closing WebSocket: {e}")))?;
        Ok(())
    }
}
