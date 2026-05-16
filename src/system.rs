//! System endpoints: health check and version info.

use crate::prelude::*;

/// Manager for system-level API endpoints.
#[derive(Clone)]
pub struct SystemManager {
    pub client: Client,
}

impl Bayse for SystemManager {
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
            config.rest_api_endpoint.to_string(),
            config.session_token,
            config.device_id,
        );
        Self { client }
    }
}

impl SystemManager {
    /// Check if the API is running.
    ///
    /// Returns `Ok(true)` on a successful response, `Ok(false)` on an API error,
    /// or `Err(BayseError)` on network/parse failures.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use bayse::prelude::*;
    ///
    /// #[tokio::main]
    /// async fn main() {
    ///     let sys = SystemManager::new(None, None);
    ///     match sys.health().await {
    ///         Ok(true) => println!("API is healthy"),
    ///         Ok(false) => println!("API returned an error"),
    ///         Err(e) => println!("Check failed: {e}"),
    ///     }
    /// }
    /// ```
    pub async fn health(&self) -> Result<HealthResponse, BayseError> {
        self.client
            .get::<HealthResponse>(API::System(SystemEndpoint::Health).as_ref(), None)
            .await
    }

    /// Get the current deployed version.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use bayse::prelude::*;
    ///
    /// #[tokio::main]
    /// async fn main() {
    ///     let sys = SystemManager::new(None, None);
    ///     match sys.version().await {
    ///         Ok(v) => println!("Version: {v:?}"),
    ///         Err(e) => println!("Failed: {e}"),
    ///     }
    /// }
    /// ```
    pub async fn version(&self) -> Result<VersionResponse, BayseError> {
        self.client
            .get::<VersionResponse>(API::System(SystemEndpoint::Version).as_ref(), None)
            .await
    }
}
