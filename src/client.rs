//! HTTP client for the Bayse Markets REST API.
//!
//! Provides public (unsigned), read (API key header), and write (HMAC-SHA256
//! signed) request methods, plus WebSocket connection helpers.

use crate::prelude::*;

/// Low-level HTTP client for the Bayse Markets API.
///
/// Wraps a `reqwest::Client` and handles authentication header injection.
#[derive(Clone)]
pub struct Client {
    /// Optional session token for session-authenticated requests.
    pub session_token: Option<String>,

    /// Optional device ID sent alongside the session token.
    pub device_id: Option<String>,

    /// Optional public API key for read-level authentication.
    pub api_key: Option<String>,

    /// Optional secret key for HMAC signing of write requests.
    pub secret_key: Option<String>,

    /// The base host URL (e.g. `https://relay.bayse.markets`).
    pub host: String,

    /// The underlying reqwest client.
    pub inner_client: ReqwestClient,
}

impl Client {
    /// Create a new `Client`.
    ///
    /// * `api_key` – Optional public API key.
    /// * `secret_key` – Optional secret key for HMAC signing.
    /// * `host` – Base URL (scheme + host).
    /// * `session_token` – Optional session token.
    /// * `device_id` – Optional device ID.
    pub fn new(
        api_key: Option<String>,
        secret_key: Option<String>,
        host: String,
        session_token: Option<String>,
        device_id: Option<String>,
    ) -> Self {
        Self {
            session_token,
            device_id,
            api_key,
            secret_key,
            host,
            inner_client: ReqwestClient::new(),
        }
    }

    // ------------------------------------------------------------------
    // Public (unsigned) requests
    // ------------------------------------------------------------------

    /// Perform a public GET request.
    ///
    /// **Auth level:** None (public endpoint, no authentication required).
    pub async fn get<T: DeserializeOwned>(
        &self,
        endpoint: &str,
        query: Option<String>,
    ) -> Result<T, BayseError> {
        let url = self.build_url(endpoint, query);
        let req = self
            .inner_client
            .get(&url)
            .header(USER_AGENT, "rs_bayse/0.1.0");
        self.handler(req).await
    }

    /// Perform a public POST request.
    ///
    /// **Auth level:** None (public endpoint, no authentication required).
    pub async fn post<T: DeserializeOwned>(
        &self,
        endpoint: &str,
        body: Option<String>,
    ) -> Result<T, BayseError> {
        let url = self.build_url(endpoint, None);
        let mut req = self
            .inner_client
            .post(&url)
            .header(USER_AGENT, "rs_bayse/0.1.0");
        if let Some(b) = body {
            req = req.header(CONTENT_TYPE, "application/json").body(b);
        }
        self.handler(req).await
    }

    /// Perform a public DELETE request.
    ///
    /// **Auth level:** None (public endpoint, no authentication required).
    pub async fn delete<T: DeserializeOwned>(
        &self,
        endpoint: &str,
        query: Option<String>,
    ) -> Result<T, BayseError> {
        let url = self.build_url(endpoint, query);
        let req = self
            .inner_client
            .delete(&url)
            .header(USER_AGENT, "rs_bayse/0.1.0");
        self.handler(req).await
    }

    // ------------------------------------------------------------------
    // Session-authenticated requests (x-auth-token + x-device-id)
    // ------------------------------------------------------------------

    /// Perform a GET request authenticated with a session token.
    ///
    /// **Auth level:** Session (`x-auth-token` + `x-device-id` headers).
    /// The token and device ID are set at construction time or after a
    /// successful [`login`](crate::UserManager::login) call.
    pub async fn get_session<T: DeserializeOwned>(
        &self,
        endpoint: &str,
        query: Option<String>,
    ) -> Result<T, BayseError> {
        let url = self.build_url(endpoint, query);
        let mut req = self
            .inner_client
            .get(&url)
            .header(USER_AGENT, "rs_bayse/0.1.0");
        req = self.inject_session_headers(req);
        self.handler(req).await
    }

    /// Perform a DELETE request authenticated with a session token.
    ///
    /// **Auth level:** Session (`x-auth-token` + `x-device-id` headers).
    pub async fn delete_session<T: DeserializeOwned>(
        &self,
        endpoint: &str,
        query: Option<String>,
    ) -> Result<T, BayseError> {
        let url = self.build_url(endpoint, query);
        let mut req = self
            .inner_client
            .delete(&url)
            .header(USER_AGENT, "rs_bayse/0.1.0");
        req = self.inject_session_headers(req);
        self.handler(req).await
    }

    /// Perform a POST request authenticated with a session token.
    ///
    /// **Auth level:** Session (`x-auth-token` + `x-device-id` headers).
    pub async fn post_session<T: DeserializeOwned>(
        &self,
        endpoint: &str,
        body: Option<String>,
    ) -> Result<T, BayseError> {
        let url = self.build_url(endpoint, None);
        let mut req = self
            .inner_client
            .post(&url)
            .header(USER_AGENT, "rs_bayse/0.1.0");
        req = self.inject_session_headers(req);
        if let Some(b) = body {
            req = req.header(CONTENT_TYPE, "application/json").body(b);
        }
        self.handler(req).await
    }

    // ------------------------------------------------------------------
    // API-key read-level requests (X-Public-Key header)
    // ------------------------------------------------------------------

    /// Perform a GET request authenticated with the public API key.
    ///
    /// **Auth level:** Read (uses `X-Public-Key` header).
    /// Requires a valid API key pair configured on the client.
    pub async fn get_read<T: DeserializeOwned>(
        &self,
        endpoint: &str,
        query: Option<String>,
    ) -> Result<T, BayseError> {
        let url = self.build_url(endpoint, query);
        let mut req = self
            .inner_client
            .get(&url)
            .header(USER_AGENT, "rs_bayse/0.1.0");
        req = self.inject_read_headers(req);
        self.handler(req).await
    }

    // ------------------------------------------------------------------
    // API-key write-level requests (HMAC-SHA256 signed)
    // ------------------------------------------------------------------

    /// Perform a POST request with HMAC-SHA256 signing.
    ///
    /// **Auth level:** Write (HMAC-SHA256 signature via `X-Public-Key`,
    /// `X-Timestamp`, and `X-Signature` headers). Requires both the
    /// public API key and the secret key configured on the client.
    pub async fn post_signed<T: DeserializeOwned>(
        &self,
        endpoint: &str,
        body: Option<String>,
    ) -> Result<T, BayseError> {
        let url = self.build_url(endpoint, None);
        let timestamp = get_timestamp().to_string();

        let payload = body.clone().unwrap_or_default();
        let signature = self.sign_message(&timestamp, endpoint, &payload);

        let mut req = self
            .inner_client
            .post(&url)
            .header(USER_AGENT, "rs_bayse/0.1.0")
            .header("X-Public-Key", self.api_key.as_deref().unwrap_or(""))
            .header("X-Timestamp", &timestamp)
            .header("X-Signature", signature);

        if let Some(b) = body {
            req = req.header(CONTENT_TYPE, "application/json").body(b);
        }
        self.handler(req).await
    }

    /// Perform a DELETE request with HMAC-SHA256 signing.
    ///
    /// **Auth level:** Write (HMAC-SHA256 signature via `X-Public-Key`,
    /// `X-Timestamp`, and `X-Signature` headers).
    pub async fn delete_signed<T: DeserializeOwned>(
        &self,
        endpoint: &str,
        query: Option<String>,
    ) -> Result<T, BayseError> {
        let url = self.build_url(endpoint, query);
        let timestamp = get_timestamp().to_string();
        let signature = self.sign_message(&timestamp, endpoint, "");

        let req = self
            .inner_client
            .delete(&url)
            .header(USER_AGENT, "rs_bayse/0.1.0")
            .header("X-Public-Key", self.api_key.as_deref().unwrap_or(""))
            .header("X-Timestamp", &timestamp)
            .header("X-Signature", signature);
        self.handler(req).await
    }

    // ------------------------------------------------------------------
    // WebSocket connection
    // ------------------------------------------------------------------

    /// Connect to a Bayse WebSocket endpoint.
    ///
    /// The scheme is automatically converted from `https://` to `wss://`
    /// (or `http://` to `ws://`) based on the client's configured host.
    ///
    /// **Auth level:** None (public connection). Per-message authentication
    /// for user channels is handled separately in the stream layer.
    ///
    /// # Returns
    ///
    /// A connected [`WebSocketStream<MaybeTlsStream<TcpStream>>`] that can
    /// be wrapped in a [`WsClient`](crate::ws::client::WsClient) for use
    /// with the high-level stream handlers.
    pub async fn wss_connect(
        &self,
        endpoint: &str,
    ) -> Result<WebSocketStream<MaybeTlsStream<TcpStream>>, BayseError> {
        let ws_url = format!(
            "{}{}",
            self.host
                .replace("https://", "wss://")
                .replace("http://", "ws://"),
            endpoint
        );
        let (stream, _) = connect_async(&ws_url).await?;
        Ok(stream)
    }

    // ------------------------------------------------------------------
    // Internal helpers
    // ------------------------------------------------------------------

    /// Build the full URL from endpoint and optional query string.
    ///
    /// If a non-empty query string is provided, it is appended after a `?`.
    /// Otherwise only `host + endpoint` is returned.
    fn build_url(&self, endpoint: &str, query: Option<String>) -> String {
        match query {
            Some(q) if !q.is_empty() => format!("{}{}?{}", self.host, endpoint, q),
            _ => format!("{}{}", self.host, endpoint),
        }
    }

    /// Inject session-level authentication headers (`x-auth-token` and `x-device-id`).
    ///
    /// Only adds headers when `session_token` / `device_id` are `Some`.
    /// If either is `None` the corresponding header is omitted.
    fn inject_session_headers(&self, req: reqwest::RequestBuilder) -> reqwest::RequestBuilder {
        let mut req = req;
        if let Some(ref token) = self.session_token {
            req = req.header("x-auth-token", token);
        }
        if let Some(ref device_id) = self.device_id {
            req = req.header("x-device-id", device_id);
        }
        req
    }

    /// Inject read-level authentication header (`X-Public-Key`).
    ///
    /// Only adds the header when `api_key` is `Some`. If the key is not
    /// configured the request is sent without any authentication, which
    /// may result in a `401 Unauthorized` from the server.
    fn inject_read_headers(&self, req: reqwest::RequestBuilder) -> reqwest::RequestBuilder {
        if let Some(ref key) = self.api_key {
            req.header("X-Public-Key", key)
        } else {
            req
        }
    }

    /// Build the HMAC-SHA256 signature for write-level requests.
    ///
    /// The signed message is formed by concatenating:
    /// `timestamp + HTTP_method + endpoint + body`.
    ///
    /// The resulting signature is hex-encoded. An empty string is used
    /// as the key material when `secret_key` is `None`, which will
    /// produce a non-functional signature (the server will reject it).
    fn sign_message(&self, timestamp: &str, endpoint: &str, body: &str) -> String {
        let secret = self.secret_key.as_deref().unwrap_or("");
        let message = format!("{}{}{}{}", timestamp, "POST", endpoint, body);

        let mut mac = Hmac::<Sha256>::new_from_slice(secret.as_bytes()).expect("HMAC key length");
        use hmac::Mac;
        mac.update(message.as_bytes());
        let result = mac.finalize();
        let code = result.into_bytes();
        hex_encode(code.as_slice())
    }

    /// Send the request and deserialise the response.
    ///
    /// On a successful HTTP status (2xx) the response body is parsed as
    /// `T`. On error, the response body is first checked for a structured
    /// [`BayseContentError`]; if that fails, an appropriate
    /// [`BayseError`] variant is returned based on the status code
    /// (401 → `Unauthorized`, 500 → `InternalServerError`, 503 →
    /// `ServiceUnavailable`, everything else → `StatusCode`).
    pub(crate) async fn handler<T: DeserializeOwned>(
        &self,
        req: reqwest::RequestBuilder,
    ) -> Result<T, BayseError> {
        let response = req.send().await?;
        let status = response.status();
        let bytes = response.bytes().await?;

        if status.is_success() {
            let value: T = serde_json::from_slice(&bytes)?;
            Ok(value)
        } else {
            // Try to parse the API error body
            let api_error: Result<BayseContentError, _> = serde_json::from_slice(&bytes);
            match api_error {
                Ok(err) => Err(BayseError::ApiError(err)),
                Err(_) => match status.as_u16() {
                    401 => Err(BayseError::Unauthorized),
                    500 => Err(BayseError::InternalServerError),
                    503 => Err(BayseError::ServiceUnavailable),
                    code => Err(BayseError::StatusCode(code)),
                },
            }
        }
    }

    /// Get a reference to the underlying reqwest client for custom usage.
    ///
    /// This allows callers to bypass the high-level methods and issue
    /// custom HTTP requests while still using the same connection pool.
    pub fn inner(&self) -> &ReqwestClient {
        &self.inner_client
    }
}
