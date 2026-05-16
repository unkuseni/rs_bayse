//! Wallet endpoints: asset balances.

use crate::prelude::*;

/// Manager for wallet-related API endpoints.
#[derive(Clone)]
pub struct WalletManager {
    pub client: Client,
}

impl Bayse for WalletManager {
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

impl WalletManager {
    /// Get wallet assets and balances for the authenticated user.
    ///
    /// Requires a valid API key (read-level: `X-Public-Key` header).
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use bayse::prelude::*;
    ///
    /// #[tokio::main]
    /// async fn main() {
    ///     let wallet = WalletManager::new(
    ///         Some("your_public_key".into()),
    ///         None,
    ///     );
    ///     match wallet.get_assets().await {
    ///         Ok(assets) => println!("Assets: {assets:#?}"),
    ///         Err(e) => println!("Failed: {e}"),
    ///     }
    /// }
    /// ```
    pub async fn get_assets(&self) -> Result<AssetResponse, BayseError> {
        self.client
            .get_read(API::Wallet(WalletEndpoint::GetAssets).as_ref(), None)
            .await
    }
}
