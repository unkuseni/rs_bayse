//! Low-level WebSocket connection wrapper.

use crate::prelude::*;

/// Manages a single WebSocket connection lifecycle.
///
/// Wraps a connected `WebSocketStream` and provides methods to send,
/// receive, and gracefully close the connection.
pub struct WsClient {
    /// The underlying WebSocket stream used for reading and writing messages.
    stream: WebSocketStream<MaybeTlsStream<TcpStream>>,
}

impl WsClient {
    /// Wrap a connected stream (typically from `Client::wss_connect`).
    pub fn new(stream: WebSocketStream<MaybeTlsStream<TcpStream>>) -> Self {
        Self { stream }
    }

    /// Get a mutable reference to the underlying stream.
    pub fn stream(&mut self) -> &mut WebSocketStream<MaybeTlsStream<TcpStream>> {
        &mut self.stream
    }

    /// Send a WebSocket Close frame and consume the connection.
    ///
    /// Sends a Close frame to the server with no status code or reason,
    /// then waits for the server's Close echo before closing the
    /// underlying TCP socket. After this call the `WsClient` is
    /// still usable for reading remaining messages, but no further
    /// sends are possible — the stream is half-closed.
    pub async fn disconnect(&mut self) -> Result<(), BayseError> {
        self.stream
            .close(None)
            .await
            .map_err(|e| BayseError::Base(format!("Error closing WebSocket: {e}")))?;
        Ok(())
    }
}
