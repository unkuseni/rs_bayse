//! High-level client facade.
//!
//! [`BayseClient`] bundles every manager into one object and abstracts the
//! multi-step authentication flows, so you don't have to wire up sessions,
//! API keys, and managers by hand.
//!
//! # Examples
//!
//! ```no_run
//! use bayse::prelude::*;
//!
//! #[tokio::main]
//! async fn main() -> Result<(), BayseError> {
//!     // Public access — no credentials needed
//!     let client = BayseClient::public();
//!     let events = client.open_events(Some(5)).await?;
//!     println!("{} open events", events.len());
//!
//!     // Full onboarding in one call: login + create API key + wire everything
//!     let (client, key) = BayseClient::login_and_create_api_key(
//!         "you@example.com",
//!         "your-password",
//!         "my-trading-bot",
//!     )
//!     .await?;
//!     println!(
//!         "Ready to trade! public_key={} secret_key={}",
//!         key.public_key, key.secret_key
//!     );
//!     Ok(())
//! }
//! ```

use crate::prelude::*;

/// A fully-wired Bayse client with every manager pre-configured.
///
/// All managers share the same credentials, so you can browse events,
/// read market data, trade, and stream prices through one object.
#[derive(Clone)]
pub struct BayseClient {
    /// Health check, version info.
    pub system: SystemManager,
    /// Login, user lookup, API key management.
    pub user: UserManager,
    /// Events, quotes, orders, portfolio, PnL, sports.
    pub trading: TradingManager,
    /// Wallet asset balances.
    pub wallet: WalletManager,
    /// Price history, order books, tickers, trades.
    pub market_data: MarketDataManager,
    /// Liquidity rewards and maker rebates.
    pub market_maker: MarketMakerManager,
    /// WebSocket streams (market data, user orders, asset prices).
    pub stream: Stream,
}

impl Bayse for BayseClient {
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
        Self {
            system: SystemManager::new_with_config(config.clone()),
            user: UserManager::new_with_config(config.clone()),
            trading: TradingManager::new_with_config(config.clone()),
            wallet: WalletManager::new_with_config(config.clone()),
            market_data: MarketDataManager::new_with_config(config.clone()),
            market_maker: MarketMakerManager::new_with_config(config.clone()),
            stream: Stream::new_with_config(config),
        }
    }
}

impl BayseClient {
    /// Create a client for public endpoints — no credentials needed.
    ///
    /// Can browse events, read market data, and subscribe to public
    /// WebSocket feeds. Authenticated endpoints will return `401`.
    pub fn public() -> Self {
        Self::new(None, None)
    }

    /// Create a client with API key credentials for read/write access.
    ///
    /// * `public_key` – Your public API key (`pk_*`).
    /// * `secret_key` – Your secret API key (`sk_*`), used for HMAC signing.
    pub fn with_api_key(public_key: impl Into<String>, secret_key: impl Into<String>) -> Self {
        Self::new(Some(public_key.into()), Some(secret_key.into()))
    }

    /// Create a client with an existing session token and device ID.
    ///
    /// Useful when you've cached credentials from a previous
    /// [`login`](Self::login) call.
    pub fn with_session(access_token: impl Into<String>, device_id: impl Into<String>) -> Self {
        let config = Config::default()
            .with_session_token(access_token.into(), device_id.into());
        Self::new_with_config(config)
    }

    /// Log in with an email and password and return a session-authenticated
    /// client.
    ///
    /// The returned client can manage API keys (create, list, revoke,
    /// rotate) and call session-authenticated endpoints. To trade, call
    /// [`login_and_create_api_key`](Self::login_and_create_api_key)
    /// instead, which also provisions an API key.
    pub async fn login(email: &str, password: &str) -> Result<Self, BayseError> {
        let mut user = UserManager::new(None, None);
        let resp = user.login(email, password).await?;
        let config = Config::default()
            .with_session_token(resp.token, resp.device_id);
        Ok(Self::new_with_config(config))
    }

    /// One-call onboarding: log in, create an API key, and return a fully
    /// authenticated client ready to trade.
    ///
    /// The returned [`CreateApiKeyResponse`] includes the secret key —
    /// store it securely, it is shown only once. The client is wired with
    /// both the session (for key management) and the new API key (for
    /// read/write trading endpoints).
    pub async fn login_and_create_api_key(
        email: &str,
        password: &str,
        key_name: &str,
    ) -> Result<(Self, CreateApiKeyResponse), BayseError> {
        let mut user = UserManager::new(None, None);
        let resp = user.login(email, password).await?;
        let key = user.create_api_key(key_name).await?;
        let config = Config::default()
            .with_session_token(resp.token, resp.device_id)
            .with_api_key(key.public_key.clone(), key.secret_key.clone());
        Ok((Self::new_with_config(config), key))
    }

    // ------------------------------------------------------------------
    // Convenience data helpers — combine steps and return clean data
    // ------------------------------------------------------------------

    /// Fetch the first page of open events.
    ///
    /// Equivalent to `trading.list_events(Some(1), size)` with the
    /// pagination wrapper stripped off.
    pub async fn open_events(&self, size: Option<u32>) -> Result<Vec<Event>, BayseError> {
        Ok(self.trading.list_events(Some(1), size).await?.events)
    }

    /// Fetch an event and return just its markets.
    ///
    /// Equivalent to `trading.get_event(event_id)` with the event wrapper
    /// stripped off.
    pub async fn markets_for(&self, event_id: &str) -> Result<Vec<Market>, BayseError> {
        Ok(self.trading.get_event(event_id).await?.markets)
    }

    /// Fetch the authenticated user's wallet asset balances.
    pub async fn balances(&self) -> Result<Vec<Asset>, BayseError> {
        Ok(self.wallet.get_assets().await?.assets)
    }
}
