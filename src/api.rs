//! API endpoint path definitions and the `Bayse` trait for creating manager
//! instances.

use crate::prelude::*;

// ---------------------------------------------------------------------------
// Endpoint enums
// ---------------------------------------------------------------------------

/// REST API endpoint groups.
#[derive(Debug, Clone)]
pub enum API {
    System(SystemEndpoint),
    User(UserEndpoint),
    Trading(TradingEndpoint),
    Wallet(WalletEndpoint),
    MarketData(MarketDataEndpoint),
    MarketMaker(MarketMakerEndpoint),
}

/// System endpoints.
#[derive(Debug, Clone)]
pub enum SystemEndpoint {
    Health,
    Version,
}

/// User endpoints.
#[derive(Debug, Clone)]
pub enum UserEndpoint {
    Lookup,
    Login,
    CreateApiKey,
    ListApiKeys,
    RevokeApiKey,
    RotateApiKey,
}

/// Trading / prediction market endpoints.
#[derive(Debug, Clone)]
pub enum TradingEndpoint {
    ListEvents,
    GetEvent,
    GetEventBySlug,
    ListSeries,
    GetSeriesEvents,
    GetQuote,
    PlaceOrder,
    BatchPlaceOrders,
    GetPortfolio,
    GetPnL,
    ListOrders,
    GetOrder,
    CancelOrder,
    BatchCancelOrders,
    MintShares,
    BurnShares,
    Activities,
}

/// Wallet endpoints.
#[derive(Debug, Clone)]
pub enum WalletEndpoint {
    GetAssets,
}

/// Market data endpoints.
#[derive(Debug, Clone)]
pub enum MarketDataEndpoint {
    PriceHistory,
    OrderBook,
    Ticker,
    Trades,
}

/// Market maker endpoints.
#[derive(Debug, Clone)]
pub enum MarketMakerEndpoint {
    LiquidityRewards,
    ActiveLiquidityRewards,
    MakerRebates,
    ActiveMakerRebates,
}

impl AsRef<str> for API {
    fn as_ref(&self) -> &str {
        match self {
            API::System(e) => e.as_ref(),
            API::User(e) => e.as_ref(),
            API::Trading(e) => e.as_ref(),
            API::Wallet(e) => e.as_ref(),
            API::MarketData(e) => e.as_ref(),
            API::MarketMaker(e) => e.as_ref(),
        }
    }
}

impl AsRef<str> for SystemEndpoint {
    fn as_ref(&self) -> &str {
        match self {
            SystemEndpoint::Health => "/health",
            SystemEndpoint::Version => "/version",
        }
    }
}

impl AsRef<str> for UserEndpoint {
    fn as_ref(&self) -> &str {
        match self {
            UserEndpoint::Lookup => "/v1/user/lookup",
            UserEndpoint::Login => "/v1/user/login",
            UserEndpoint::CreateApiKey => "/v1/user/me/api-keys",
            UserEndpoint::ListApiKeys => "/v1/user/me/api-keys",
            UserEndpoint::RevokeApiKey => "/v1/user/me/api-keys/", // + {keyId}
            UserEndpoint::RotateApiKey => "/v1/user/me/api-keys/", // + {keyId}/rotate
        }
    }
}

impl AsRef<str> for TradingEndpoint {
    fn as_ref(&self) -> &str {
        match self {
            TradingEndpoint::ListEvents => "/v1/pm/events",
            TradingEndpoint::GetEvent => "/v1/pm/events/", // + {eventId}
            TradingEndpoint::GetEventBySlug => "/v1/pm/events/slug/", // + {slug}
            TradingEndpoint::ListSeries => "/v1/pm/events/series",
            TradingEndpoint::GetSeriesEvents => "/v1/pm/events/series/", // + {seriesSlug}/lean-events
            TradingEndpoint::GetQuote => "/v1/pm/events/", // + {eventId}/markets/{marketId}/quote
            TradingEndpoint::PlaceOrder => "/v1/pm/events/", // + {eventId}/markets/{marketId}/orders
            TradingEndpoint::BatchPlaceOrders => "/v1/pm/orders/batch",
            TradingEndpoint::GetPortfolio => "/v1/pm/portfolio",
            TradingEndpoint::GetPnL => "/v1/pm/pnl",
            TradingEndpoint::ListOrders => "/v1/pm/orders",
            TradingEndpoint::GetOrder => "/v1/pm/orders/", // + {orderId}
            TradingEndpoint::CancelOrder => "/v1/pm/orders/", // + {orderId}
            TradingEndpoint::BatchCancelOrders => "/v1/pm/orders/batch",
            TradingEndpoint::MintShares => "/v1/pm/markets/", // + {marketId}/mint
            TradingEndpoint::BurnShares => "/v1/pm/markets/", // + {marketId}/burn
            TradingEndpoint::Activities => "/v1/pm/activities",
        }
    }
}

impl AsRef<str> for WalletEndpoint {
    fn as_ref(&self) -> &str {
        match self {
            WalletEndpoint::GetAssets => "/v1/wallet/assets",
        }
    }
}

impl AsRef<str> for MarketDataEndpoint {
    fn as_ref(&self) -> &str {
        match self {
            MarketDataEndpoint::PriceHistory => "/v1/pm/events/", // + {eventId}/price-history
            MarketDataEndpoint::OrderBook => "/v1/pm/books",
            MarketDataEndpoint::Ticker => "/v1/pm/markets/", // + {marketId}/ticker
            MarketDataEndpoint::Trades => "/v1/pm/trades",
        }
    }
}

impl AsRef<str> for MarketMakerEndpoint {
    fn as_ref(&self) -> &str {
        match self {
            MarketMakerEndpoint::LiquidityRewards => "/v1/pm/liquidity-rewards",
            MarketMakerEndpoint::ActiveLiquidityRewards => "/v1/pm/liquidity-rewards/active",
            MarketMakerEndpoint::MakerRebates => "/v1/pm/maker-rebates",
            MarketMakerEndpoint::ActiveMakerRebates => "/v1/pm/maker-rebates/active",
        }
    }
}

// ---------------------------------------------------------------------------
// The `Bayse` trait – factory for manager structs
// ---------------------------------------------------------------------------

/// Trait that every manager struct implements.
///
/// Provides two constructors:
/// - `new()` – default (production, no auth)
/// - `new_with_config()` – custom config
pub trait Bayse {
    /// Create a new manager instance with optional API credentials.
    ///
    /// Uses the production endpoint. Pass `None` for public-only access.
    fn new(api_key: Option<String>, secret_key: Option<String>) -> Self;

    /// Create a new manager instance with a full `Config`.
    fn new_with_config(config: Config) -> Self;
}
