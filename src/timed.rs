//! Wrapper that pairs data with a local reception timestamp.

/// Wraps a value with the local timestamp when it was received.
///
/// Used by WebSocket stream handlers to attach reception timestamps to
/// incoming messages so callers can measure transport latency or order
/// events by arrival time.
///
/// # Type Parameters
///
/// * `T` – The type of the received data payload.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Timed<T> {
    /// The local timestamp (milliseconds since Unix epoch) when the data
    /// was received by the client.
    pub time: u64,

    /// The actual data payload received from the WebSocket stream.
    pub data: T,
}
