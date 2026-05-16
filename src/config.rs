//! Configuration for the Bayse Markets API client.

use std::borrow::Cow;

/// Client configuration holding endpoint URLs and request settings.
#[derive(Clone, Debug)]
pub struct Config {
    /// Base URL for REST API requests.
    pub rest_api_endpoint: Cow<'static, str>,

    /// Base URL for WebSocket connections.
    pub ws_endpoint: Cow<'static, str>,

    /// Session token for authenticated endpoints (optional).
    pub session_token: Option<String>,

    /// Device ID used alongside session token.
    pub device_id: Option<String>,

    /// Public API key for read/write authenticated endpoints.
    pub api_key: Option<String>,

    /// Secret API key for HMAC signing of write requests.
    pub secret_key: Option<String>,
}

impl Config {
    /// Production REST API base URL.
    pub const DEFAULT_REST_API_ENDPOINT: &'static str = "https://relay.bayse.markets";

    /// Production WebSocket base URL.
    pub const DEFAULT_WS_ENDPOINT: &'static str = "wss://socket.bayse.markets";

    /// Create a new configuration with custom endpoints.
    pub fn new(rest_api_endpoint: impl AsRef<str>, ws_endpoint: impl AsRef<str>) -> Self {
        Self {
            rest_api_endpoint: Cow::Owned(rest_api_endpoint.as_ref().to_string()),
            ws_endpoint: Cow::Owned(ws_endpoint.as_ref().to_string()),
            session_token: None,
            device_id: None,
            api_key: None,
            secret_key: None,
        }
    }

    /// Default production configuration.
    pub const fn default() -> Self {
        Self {
            rest_api_endpoint: Cow::Borrowed(Self::DEFAULT_REST_API_ENDPOINT),
            ws_endpoint: Cow::Borrowed(Self::DEFAULT_WS_ENDPOINT),
            session_token: None,
            device_id: None,
            api_key: None,
            secret_key: None,
        }
    }

    /// Builder-style: set the session token.
    pub fn with_session_token(mut self, token: String, device_id: String) -> Self {
        self.session_token = Some(token);
        self.device_id = Some(device_id);
        self
    }

    /// Builder-style: set API key pair for HMAC authentication.
    pub fn with_api_key(mut self, api_key: String, secret_key: String) -> Self {
        self.api_key = Some(api_key);
        self.secret_key = Some(secret_key);
        self
    }
}
