//! Error types for the Bayse Markets API client.
//!
//! Defines `BayseContentError` for API-level errors and `BayseError` for
//! client-level failures (network, serialisation, validation, etc.).

use crate::prelude::*;

/// Error payload returned by the Bayse Markets API.
///
/// All error responses follow a consistent JSON shape:
///
/// ```json
/// {
///   "error": "error_code",
///   "message": "Human-readable description",
///   "statusCode": 400
/// }
/// ```
#[derive(Debug, Deserialize, Display)]
#[display("{}: {}", error, message)]
pub struct BayseContentError {
    /// The machine-readable error code.
    pub error: String,

    /// A human-readable description of the error.
    pub message: String,

    /// The HTTP status code.
    #[serde(default)]
    pub status_code: u16,
}

/// Top-level error enum for the Bayse client.
///
/// Covers API errors, transport errors, and validation failures.
#[derive(Debug, Error)]
pub enum BayseError {
    /// An error returned by the Bayse API (non-2xx status).
    #[error("Bayse API error: {0}")]
    ApiError(BayseContentError),

    /// Failed to send an item through a channel.
    #[error("Failed to emit value on channel, underlying: {underlying}")]
    ChannelSendError { underlying: String },

    /// A request parameter failed validation before being sent.
    #[error("Validation error: {0}")]
    ValidationError(String),

    /// Wraps a `reqwest` transport error.
    #[error(transparent)]
    ReqwestError(#[from] reqwest::Error),

    /// Wraps an invalid header value.
    #[error(transparent)]
    InvalidHeaderError(#[from] reqwest::header::InvalidHeaderValue),

    /// Wraps an I/O error.
    #[error(transparent)]
    IoError(#[from] std::io::Error),

    /// Wraps a float parsing error.
    #[error(transparent)]
    ParseFloatError(#[from] std::num::ParseFloatError),

    /// Wraps a URL parsing error.
    #[error(transparent)]
    UrlParserError(#[from] url::ParseError),

    /// Wraps a JSON serialisation/deserialisation error.
    #[error(transparent)]
    JsonError(#[from] serde_json::Error),

    /// Wraps a WebSocket (tungstenite) error.
    #[error(transparent)]
    TungsteniteError(#[from] tokio_tungstenite::tungstenite::Error),

    /// Wraps a system timestamp error.
    #[error(transparent)]
    TimestampError(#[from] std::time::SystemTimeError),

    /// Wraps a Serde value error.
    #[error(transparent)]
    SerdeError(#[from] serde::de::value::Error),

    /// Catch-all for HTTP 500 responses.
    #[error("Internal Server Error")]
    InternalServerError,

    /// Catch-all for HTTP 503 responses.
    #[error("Service Unavailable")]
    ServiceUnavailable,

    /// Catch-all for HTTP 401 responses.
    #[error("Unauthorized")]
    Unauthorized,

    /// An unexpected HTTP status code.
    #[error("Unexpected status code: {0}")]
    StatusCode(u16),

    /// A generic string-based error.
    #[error("{0}")]
    Base(String),
}

impl From<String> for BayseError {
    fn from(err: String) -> Self {
        BayseError::Base(err)
    }
}

impl BayseError {
    pub fn new(arg: impl Into<String>) -> Self {
        BayseError::Base(arg.into())
    }
}
