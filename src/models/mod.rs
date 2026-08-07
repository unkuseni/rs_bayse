//! Data models for the Bayse Markets API.
//!
//! These are placeholder / structural types. The actual API shapes should be
//! modelled with `serde` derives as the SDK evolves. For now, most endpoints
//! return `serde_json::Value` and users can deserialise into their own types.

use crate::prelude::*;

/// Generic pagination wrapper returned by list endpoints.
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Pagination {
    /// Current page number (1-indexed).
    pub page: u32,
    /// Number of items per page.
    pub size: u32,
    /// The last available page number.
    pub last_page: u32,
    /// Total number of items across all pages.
    pub total_count: u32,
}

/// A generic API response that wraps a payload with pagination.
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct PaginatedResponse<T> {
    /// The response payload items.
    pub data: Vec<T>,
    /// Pagination metadata for the response.
    pub pagination: Pagination,
}

/// Subscription payload for WebSocket channels.
///
/// Use the builder methods to set channel-specific fields. Only the
/// fields relevant to the target channel need to be populated.
///
/// # Examples
///
/// ```
/// use bayse::WsSubscription;
///
/// // Subscribe to price updates for an event
/// let sub = WsSubscription::new("subscribe", "prices")
///     .with_event_id("event_123");
///
/// // Subscribe to order book for multiple markets
/// let ob = WsSubscription::new("subscribe", "orderbook")
///     .with_market_ids(vec!["mkt_1".into(), "mkt_2".into()])
///     .with_currency("USD");
///
/// // Subscribe to asset prices
/// let prices = WsSubscription::new("subscribe", "asset_prices")
///     .with_symbols(vec!["BTCUSDT".into()]);
/// ```
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WsSubscription {
    /// The operation type: `"subscribe"`, `"unsubscribe"`, or `"ping"`.
    #[serde(rename = "type")]
    pub op: String,

    /// The channel to subscribe to
    /// (e.g. `"activity"`, `"prices"`, `"orderbook"`, `"orders"`, `"asset_prices"`).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub channel: Option<String>,

    /// Event UUID filter. Required for `activity` and `prices` channels.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub event_id: Option<String>,

    /// Single market UUID filter. Optional for `activity`.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub market_id: Option<String>,

    /// Market UUIDs (max 10 per message). Required for `orderbook` and `orders`.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub market_ids: Option<Vec<String>>,

    /// Asset symbols (e.g. `["BTCUSDT"]`). Required for `asset_prices`.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub symbols: Option<Vec<String>>,

    /// Currency filter: `"USD"` or `"NGN"`. Optional for `orderbook`.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub currency: Option<String>,

    /// Room name to unsubscribe from. Required for `unsubscribe`.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub room: Option<String>,

    /// User UUID filter. Required for `user_trades` channel.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub user_id: Option<String>,

    /// Per-message authentication credentials. Required for `/ws/v1/user`.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub auth: Option<WsAuth>,
}

impl WsSubscription {
    /// Create a new subscription message for a given channel.
    pub fn new(op: impl Into<String>, channel: impl Into<String>) -> Self {
        Self {
            op: op.into(),
            channel: Some(channel.into()),
            event_id: None,
            market_id: None,
            market_ids: None,
            symbols: None,
            currency: None,
            room: None,
            user_id: None,
            auth: None,
        }
    }

    /// Set the event ID filter (used with `activity` and `prices` channels).
    pub fn with_event_id(mut self, event_id: impl Into<String>) -> Self {
        self.event_id = Some(event_id.into());
        self
    }

    /// Set a single market ID filter (used with `activity` channel).
    pub fn with_market_id(mut self, market_id: impl Into<String>) -> Self {
        self.market_id = Some(market_id.into());
        self
    }

    /// Set market IDs (used with `orderbook` and `orders` channels).
    pub fn with_market_ids(mut self, market_ids: Vec<String>) -> Self {
        self.market_ids = Some(market_ids);
        self
    }

    /// Set asset symbols (used with `asset_prices` channel).
    pub fn with_symbols(mut self, symbols: Vec<String>) -> Self {
        self.symbols = Some(symbols);
        self
    }

    /// Set the currency filter (used with `orderbook` channel).
    pub fn with_currency(mut self, currency: impl Into<String>) -> Self {
        self.currency = Some(currency.into());
        self
    }

    /// Set the room name (used with `unsubscribe`).
    pub fn with_room(mut self, room: impl Into<String>) -> Self {
        self.room = Some(room.into());
        self
    }

    /// Set the user ID (used with `user_trades` channel).
    pub fn with_user_id(mut self, user_id: impl Into<String>) -> Self {
        self.user_id = Some(user_id.into());
        self
    }

    /// Attach authentication credentials (required for `/ws/v1/user`).
    pub fn with_auth(mut self, auth: WsAuth) -> Self {
        self.auth = Some(auth);
        self
    }

    /// Create a `ping` message for connection keepalive.
    pub fn ping() -> Self {
        Self {
            op: "ping".into(),
            channel: None,
            event_id: None,
            market_id: None,
            market_ids: None,
            symbols: None,
            currency: None,
            room: None,
            user_id: None,
            auth: None,
        }
    }

    /// Create an `unsubscribe` message for a specific room.
    pub fn unsubscribe(room: impl Into<String>) -> Self {
        Self {
            op: "unsubscribe".into(),
            channel: None,
            event_id: None,
            market_id: None,
            market_ids: None,
            symbols: None,
            currency: None,
            room: Some(room.into()),
            user_id: None,
            auth: None,
        }
    }
}

/// Empty placeholder for API responses with no data.
#[derive(Serialize, Default, Deserialize, Clone, Debug)]
pub struct Empty {}

/// Per-message authentication credentials for the `/ws/v1/user` endpoint.
///
/// Every message sent to the user WebSocket endpoint must include one of
/// `api_key` or `access_token`. If both are provided, the access token
/// takes precedence. The server caches the last verified credential per
/// connection so repeated messages with the same value skip the auth call.
///
/// # Examples
///
/// ```
/// use bayse::WsAuth;
///
/// // Authenticate with an API key
/// let auth = WsAuth::with_api_key("pk_live_...");
///
/// // Authenticate with an access token
/// let auth = WsAuth::with_access_token("eyJ...", Some("device-123".to_string()));
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WsAuth {
    /// Public API key for relay trading clients.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub api_key: Option<String>,

    /// JWT access token. Takes precedence over `api_key` when both are set.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub access_token: Option<String>,

    /// Optional device identifier sent alongside the access token.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub device_id: Option<String>,
}

impl WsAuth {
    /// Create auth credentials from an API public key.
    pub fn with_api_key(api_key: impl Into<String>) -> Self {
        Self {
            api_key: Some(api_key.into()),
            access_token: None,
            device_id: None,
        }
    }

    /// Create auth credentials from a JWT access token, optionally with a device ID.
    pub fn with_access_token(access_token: impl Into<String>, device_id: Option<String>) -> Self {
        Self {
            api_key: None,
            access_token: Some(access_token.into()),
            device_id,
        }
    }
}

/// A generic API response wrapper used by several endpoints.
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct BayseApiResponse<T = Value> {
    pub data: T,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct HealthResponse {
    /// API health status (e.g. `"ok"`).
    pub status: String,
}

impl std::fmt::Display for HealthResponse {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.status)
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct VersionResponse {
    /// Git commit hash of the deployed build.
    pub commit: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct LoginResponse {
    /// JWT authentication token.
    pub token: String,
    /// UUID of the authenticated device.
    pub device_id: String,
    /// UUID of the authenticated user.
    pub user_id: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct CreateApiKeyResponse {
    /// UUID of the newly created API key.
    pub id: String,
    /// Human-readable label for the API key.
    pub name: String,
    /// ISO 8601 timestamp of creation.
    pub created_at: String,
    /// The public key portion of the API key pair.
    pub public_key: String,
    /// The secret key portion (shown only once).
    pub secret_key: String,
    /// Instructions for signing requests with this key.
    pub signing_instructions: SigningInstructions,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct SigningInstructions {
    /// Signing algorithm (e.g. `"ED25519"`).
    pub algorithm: String,
    /// List of header names included in the signature.
    pub headers: Vec<String>,
    /// Format of the payload to sign (e.g. `"RAW"`).
    pub payload_format: String,
    /// Allowed timestamp drift in seconds.
    pub timestamp_window_seconds: u16,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ListApiKeysResponse {
    /// List of API keys for the authenticated user.
    pub keys: Vec<ApiKey>,
    /// Total number of API keys.
    pub total: u64,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ApiKey {
    /// ISO 8601 timestamp of creation.
    pub created_at: String,
    /// UUID of the API key.
    pub id: String,
    /// Human-readable label for the API key.
    pub name: String,
    /// The public key portion of the API key pair.
    pub public_key: String,
    /// Last few characters of the secret key for identification.
    pub secret_key_hint: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct RevokeApiKeyResponse {
    /// Confirmation message for the revocation.
    pub message: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct AssetResponse {
    /// List of assets for the authenticated user.
    pub assets: Vec<Asset>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Asset {
    /// Blockchain addresses associated with the asset.
    pub addresses: Vec<Value>,
    /// Available balance in the asset's currency units.
    pub available_balance: f64,
    /// ISO 8601 timestamp when the asset was created.
    pub created_at: String,
    /// Multiplier for converting to base units (1 for USD, 100 for NGN).
    pub currency_base_multiplier: u16,
    /// Status of recent deposit activity.
    pub deposit_activity: String,
    /// UUID of the asset record.
    pub id: String,
    /// Whether this is the user's default asset.
    pub is_default: bool,
    /// Whether this is the user's local currency asset.
    pub is_local_currency_asset: bool,
    /// Blockchain network for the asset (e.g. `"ethereum"`).
    pub network: String,
    /// Balance pending settlement.
    pub pending_balance: f64,
    /// Currency symbol (e.g. `"USD"`, `"NGN"`).
    pub symbol: String,
    /// ISO 8601 timestamp when the asset was last updated.
    pub updated_at: String,
    /// UUID of the owning user.
    pub user_id: String,
    /// Status of recent wager activity.
    pub wager_activity: String,
    /// Status of recent withdrawal activity.
    pub withdrawal_activity: String,
}

// --------------------------------------------------------------------------
// Trading / prediction market types
// --------------------------------------------------------------------------

/// Response from the list-events endpoint.
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ListEventsResponse {
    /// List of prediction market events.
    pub events: Vec<Event>,
    /// Pagination metadata for the response.
    pub pagination: Pagination,
}

/// A prediction market event.
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Event {
    /// UUID of the event.
    pub id: String,
    /// Human-readable title of the event.
    pub title: String,
    /// URL-friendly identifier for the event.
    pub slug: String,
    /// Detailed description of the event.
    pub description: String,
    /// Current status of the event (e.g. `"open"`, `"closed"`, `"resolved"`).
    pub status: String,
    /// Type of the event (e.g. `"WINNER_TAKES_ALL"`).
    #[serde(rename = "type")]
    pub event_type: String,
    /// Category of the event.
    pub category: String,
    /// Trading engine used (`"AMM"` or `"CLOB"`).
    pub engine: String,
    /// ISO 8601 timestamp when the event was created.
    pub created_at: String,
    /// ISO 8601 timestamp when the event closes for trading.
    #[serde(default)]
    pub closing_date: String,
    /// ISO 8601 timestamp when the event is scheduled to resolve.
    #[serde(default)]
    pub resolution_date: String,
    /// ISO 8601 timestamp when the event was actually resolved.
    #[serde(default)]
    pub resolved_at: String,
    /// Source URL or description for the resolution.
    #[serde(default)]
    pub resolution_source: String,
    /// Additional information about the event.
    #[serde(default)]
    pub additional_context: String,
    /// Type of countdown display for the event.
    #[serde(default)]
    pub countdown_type: String,
    /// Threshold value for event resolution, if applicable.
    #[serde(default)]
    pub event_threshold: Option<f64>,
    /// Asset symbol pair associated with the event, if any.
    #[serde(default)]
    pub asset_symbol_pair: Option<String>,
    /// List of hashtags associated with the event.
    #[serde(default)]
    pub hashtags: Vec<String>,
    /// URL of the event's main image.
    #[serde(default)]
    pub image_url: String,
    /// URL of the event's 128x128 thumbnail image.
    #[serde(default)]
    pub image128_url: String,
    /// Total liquidity in the event's markets.
    pub liquidity: f64,
    /// Total number of orders placed in the event.
    pub total_orders: u64,
    /// Total trading volume across all markets in the event.
    #[serde(default)]
    pub total_volume: f64,
    /// Whether the authenticated user has watchlisted this event.
    #[serde(default)]
    pub user_watchlisted: bool,
    /// List of supported currency symbols (e.g. `"USD"`, `"NGN"`).
    #[serde(default)]
    pub supported_currencies: Vec<String>,
    /// Whether to display a countdown timer for the event.
    #[serde(default)]
    pub display_countdown: bool,
    /// List of ISO country codes where the event is available.
    #[serde(default)]
    pub country_codes: Option<Vec<String>>,
    /// Additional region-based restrictions.
    #[serde(default)]
    pub regions: Option<Value>,
    /// Optional metadata associated with the event.
    #[serde(default)]
    pub metadata: Option<Value>,
    /// Slug of the series this event belongs to, if any.
    #[serde(default)]
    pub series_slug: Option<String>,
    /// List of markets within this event.
    #[serde(default)]
    pub markets: Vec<Market>,
}

/// A market within a prediction event.
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Market {
    /// UUID of the market.
    pub id: String,
    /// Title of the market (e.g. `"Will X happen?"`).
    pub title: String,
    /// Current status of the market (e.g. `"open"`, `"resolved"`).
    pub status: String,
    /// Trading fee percentage for this market.
    #[serde(default)]
    pub fee_percentage: f64,
    /// UUID of the first outcome.
    pub outcome1_id: String,
    /// Label for the first outcome (e.g. `"Yes"`).
    pub outcome1_label: String,
    /// Current price of the first outcome (0.00–1.00).
    pub outcome1_price: f64,
    /// UUID of the second outcome.
    pub outcome2_id: String,
    /// Label for the second outcome (e.g. `"No"`).
    pub outcome2_label: String,
    /// Current price of the second outcome (0.00–1.00).
    pub outcome2_price: f64,
    /// Price to buy the `"Yes"` outcome.
    #[serde(default)]
    pub yes_buy_price: f64,
    /// Price to buy the `"No"` outcome.
    #[serde(default)]
    pub no_buy_price: f64,
    /// Estimated `"Yes"` price used for profit calculations.
    #[serde(default)]
    pub yes_price_for_estimate: f64,
    /// Estimated profit percentage for the `"Yes"` outcome.
    #[serde(default)]
    pub yes_profit_for_estimate: f64,
    /// Estimated `"No"` price used for profit calculations.
    #[serde(default)]
    pub no_price_for_estimate: f64,
    /// Estimated profit percentage for the `"No"` outcome.
    #[serde(default)]
    pub no_profit_for_estimate: f64,
    /// Final resolution price of the market.
    #[serde(default)]
    pub resolution_price: f64,
    /// The outcome that won upon resolution (e.g. `"YES"` or `"NO"`).
    #[serde(default)]
    pub resolved_outcome: String,
    /// Threshold value for market resolution, if applicable.
    #[serde(default)]
    pub market_threshold: Option<f64>,
    /// Rules text for the market.
    #[serde(default)]
    pub rules: String,
    /// URL of the market's image.
    #[serde(default)]
    pub image_url: String,
    /// URL of the market's 128x128 thumbnail image.
    #[serde(default)]
    pub image128_url: String,
    /// Total number of orders placed in this market.
    #[serde(default)]
    pub total_orders: u64,
    /// Liquidity reward configuration, if any.
    #[serde(default)]
    pub liquidity_reward: Option<LiquidityReward>,
    /// Maker rebate configuration, if any.
    #[serde(default)]
    pub maker_rebate: Option<MakerRebate>,
}

/// Liquidity reward configuration for a market.
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct LiquidityReward {
    /// Total reward pool for liquidity providers.
    pub reward_pool: f64,
    /// Maximum allowable spread in cents for the reward.
    pub max_spread_cents: u64,
    /// Minimum notional order size to qualify for the reward.
    pub min_notional_order_size: f64,
}

/// Maker rebate configuration for a market.
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct MakerRebate {
    /// UUID of the maker rebate configuration.
    pub config_id: String,
    /// Percentage of fees rebated to makers.
    pub rebate_percentage: f64,
    /// Minimum payout in USD for the rebate.
    pub min_payout_usd: f64,
}

// --------------------------------------------------------------------------
// Market maker reward / rebate response types
// --------------------------------------------------------------------------

/// A liquidity reward payout record from get-liquidity-rewards.
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct LiquidityRewardRecord {
    /// UUID of the reward epoch.
    pub epoch_id: String,
    /// UUID of the associated event.
    pub event_id: String,
    /// UUID of the associated market.
    pub market_id: String,
    /// Total shares accumulated during the epoch.
    pub accumulated_shares: f64,
    /// Number of samples taken during the epoch.
    pub sample_count: u64,
    /// Payout amount received for the epoch.
    pub payout: f64,
    /// Whether the payout has been disbursed.
    pub is_paid: bool,
    /// ISO 8601 timestamp of the epoch start.
    pub epoch_start: String,
    /// ISO 8601 timestamp of the epoch end.
    pub epoch_end: String,
    /// Status of the reward record (e.g. `"active"`, `"paid"`).
    pub status: String,
}

/// Paginated response from get-liquidity-rewards.
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct LiquidityRewardsResponse {
    /// List of liquidity reward records.
    pub data: Vec<LiquidityRewardRecord>,
    /// Pagination metadata for the response.
    pub pagination: Pagination,
}

/// An active (in-progress) liquidity reward from get-active-liquidity-rewards.
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ActiveLiquidityReward {
    /// UUID of the current reward epoch.
    pub epoch_id: String,
    /// UUID of the associated event.
    pub event_id: String,
    /// UUID of the associated market.
    pub market_id: String,
    /// Shares accumulated so far in the epoch.
    pub accumulated_shares: f64,
    /// Number of samples recorded so far.
    pub sample_count: u64,
    /// Estimated payout for the current epoch.
    pub estimated_payout: f64,
    /// ISO 8601 timestamp of the epoch start.
    pub epoch_start: String,
    /// ISO 8601 timestamp of the epoch end.
    pub epoch_end: String,
}

/// Response from get-active-liquidity-rewards.
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ActiveLiquidityRewardsResponse {
    /// List of active liquidity rewards.
    pub data: Vec<ActiveLiquidityReward>,
}

/// A maker rebate payout record from get-maker-rebates.
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct MakerRebateRecord {
    /// UUID of the rebate epoch.
    pub epoch_id: String,
    /// UUID of the associated event.
    pub event_id: String,
    /// UUID of the associated market.
    pub market_id: String,
    /// Total maker volume during the epoch.
    pub maker_volume: f64,
    /// Number of qualifying trades during the epoch.
    pub trade_count: u64,
    /// Rebate amount earned for the epoch.
    pub rebate_amount: f64,
    /// Whether the rebate has been disbursed.
    pub is_paid: bool,
    /// ISO 8601 timestamp of the epoch start.
    pub epoch_start: String,
    /// ISO 8601 timestamp of the epoch end.
    pub epoch_end: String,
    /// Status of the rebate record.
    pub status: String,
}

/// Paginated response from get-maker-rebates.
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct MakerRebatesResponse {
    /// List of maker rebate records.
    pub data: Vec<MakerRebateRecord>,
    /// Pagination metadata for the response.
    pub pagination: Pagination,
}

/// An active (in-progress) maker rebate from get-active-maker-rebates.
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ActiveMakerRebate {
    /// UUID of the current rebate epoch.
    pub epoch_id: String,
    /// UUID of the associated event.
    pub event_id: String,
    /// UUID of the associated market.
    pub market_id: String,
    /// Maker volume accumulated so far in the epoch.
    pub maker_volume: f64,
    /// Number of trades recorded so far.
    pub trade_count: u64,
    /// Rebate amount accumulated so far.
    pub rebate_amount: f64,
    /// ISO 8601 timestamp of the epoch start.
    pub epoch_start: String,
    /// ISO 8601 timestamp of the epoch end.
    pub epoch_end: String,
}

/// Response from get-active-maker-rebates.
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ActiveMakerRebatesResponse {
    /// List of active maker rebates.
    pub data: Vec<ActiveMakerRebate>,
}

// --------------------------------------------------------------------------
// Quote types
// --------------------------------------------------------------------------

/// Request body for the get-quote endpoint.
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct GetQuoteRequest {
    /// `"BUY"` or `"SELL"`.
    pub side: String,
    /// UUID of the outcome to quote.
    pub outcome_id: String,
    /// Amount to spend (buy) or receive (sell) in the specified currency.
    pub amount: f64,
    /// `"USD"` (default) or `"NGN"`.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub currency: Option<String>,
}

// --------------------------------------------------------------------------
// Order types
// --------------------------------------------------------------------------

/// Request body for the place-order endpoint.
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct PlaceOrderRequest {
    /// `"BUY"` or `"SELL"`.
    pub side: String,
    /// UUID of the outcome to trade.
    pub outcome_id: String,
    /// Amount to spend (buy) or receive (sell). Minimum $1 USD / ₦100 NGN.
    pub amount: f64,
    /// `"LIMIT"` or `"MARKET"`.
    #[serde(rename = "type")]
    pub order_type: String,
    /// `"USD"` (default) or `"NGN"`.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub currency: Option<String>,
    /// Limit price per share (0.01–0.99). Required for LIMIT orders.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub price: Option<f64>,
    /// `"GTC"` (default for limit), `"GTD"`, `"FAK"` (default for market),
    /// or `"FOK"`.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub time_in_force: Option<String>,
    /// If `true`, the order is rejected instead of crossing the spread.
    /// Limit orders only. Default: `false`.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub post_only: Option<bool>,
    /// Self-trade prevention mode. CLOB only. Default: `"SKIP"`.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stp_mode: Option<String>,
    /// Maximum acceptable slippage for market orders (0.00–1.00).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_slippage: Option<f64>,
    /// ISO 8601 expiration timestamp. Required for `GTD` orders.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub expires_at: Option<String>,
}

/// Response from the place-order endpoint.
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct PlaceOrderResponse {
    /// `"AMM"` or `"CLOB"`.
    pub engine: String,
    /// The placed order (fields vary by engine type).
    pub order: PlacedOrder,
}

/// An order returned from the place-order endpoint.
///
/// Common fields are required; CLOB-specific fields are `Option` and
/// default to `None` when absent (AMM response).
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct PlacedOrder {
    /// UUID of the order.
    pub id: String,
    /// `"BUY"` or `"SELL"`.
    pub side: String,
    /// Total amount spent (or received for sells).
    pub amount: f64,
    /// Average fill price per share.
    pub price: f64,
    /// Number of shares received.
    pub quantity: f64,
    /// Order status: `"filled"`, `"open"`, `"partial_filled"`, `"cancelled"`, etc.
    pub status: String,
    /// Currency used (`"USD"` or `"NGN"`).
    pub currency: String,
    /// ISO 8601 timestamp of creation.
    pub created_at: String,
    /// ISO 8601 timestamp of last update.
    pub updated_at: String,
    /// `"YES"` or `"NO"`.
    #[serde(default)]
    pub outcome: Option<String>,
    /// `"LIMIT"`, `"MARKET"`, etc.
    #[serde(rename = "type")]
    pub order_type: String,
    // --- CLOB-specific extras ---
    #[serde(default)]
    pub market_id: Option<String>,
    #[serde(default)]
    pub user_id: Option<String>,
    #[serde(default)]
    pub order_type_clob: Option<String>,
    #[serde(default)]
    pub size: Option<f64>,
    #[serde(default)]
    pub filled_size: Option<f64>,
    #[serde(default)]
    pub remaining_size: Option<f64>,
    #[serde(default)]
    pub avg_fill_price: Option<f64>,
    #[serde(default)]
    pub fee: Option<f64>,
    #[serde(default)]
    pub post_only: Option<bool>,
    #[serde(default)]
    pub stp_mode: Option<String>,
}

// --------------------------------------------------------------------------
// Batch order types
// --------------------------------------------------------------------------

/// A single order item in a batch-place request.
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct BatchOrderItem {
    /// UUID of the outcome to trade.
    pub outcome_id: String,
    /// `"BUY"` or `"SELL"`.
    pub side: String,
    /// `"LIMIT"` or `"MARKET"`.
    #[serde(rename = "type")]
    pub order_type: String,
    /// Amount to spend (buy) or receive (sell).
    pub amount: f64,
    /// `"USD"` (default) or `"NGN"`.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub currency: Option<String>,
    /// Limit price per share (0.01–0.99). Required for LIMIT orders.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub price: Option<f64>,
    /// `"GTC"`, `"GTD"`, `"FAK"`, or `"FOK"`.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub time_in_force: Option<String>,
    /// If `true`, the order is rejected instead of crossing the spread.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub post_only: Option<bool>,
    /// Maximum acceptable slippage for market orders (0.00–1.00).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_slippage: Option<f64>,
    /// ISO 8601 expiration timestamp. Required for `GTD` orders.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub expires_at: Option<String>,
    /// Self-trade prevention mode.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stp_mode: Option<String>,
    /// Optional client-supplied identifier echoed back in the result.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub client_order_id: Option<String>,
}

/// Request body for the batch-place-orders endpoint.
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct BatchPlaceOrdersRequest {
    /// 1–20 order items.
    pub orders: Vec<BatchOrderItem>,
}

/// Per-order outcome in a batch response.
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct BatchOrderResult {
    /// Position in the request array (zero-based).
    pub index: u32,
    /// Echoed from the request, if provided.
    #[serde(default)]
    pub client_order_id: Option<String>,
    /// `true` if the order was accepted.
    pub success: bool,
    /// The placed CLOB order. Present when `success` is `true`.
    #[serde(default)]
    pub order: Option<Value>,
    /// Error details. Present when `success` is `false`.
    #[serde(default)]
    pub error: Option<BatchOrderError>,
}

/// Error details for a failed batch order item.
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct BatchOrderError {
    /// Machine-readable error code.
    pub code: String,
    /// Human-readable description.
    pub message: String,
}

/// Summary of a batch-place response.
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct BatchSummary {
    pub total: u32,
    pub succeeded: u32,
    pub failed: u32,
}

/// Response from the batch-place-orders endpoint.
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct BatchPlaceOrdersResponse {
    /// Always `"CLOB"` for batch endpoints.
    pub engine: String,
    /// Per-order outcomes in the same order as the request.
    pub results: Vec<BatchOrderResult>,
    /// Aggregate counts.
    pub summary: BatchSummary,
}

// --------------------------------------------------------------------------
// Order list / detail types
// --------------------------------------------------------------------------

/// An order returned from list-orders or get-order.
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Order {
    /// UUID of the order.
    pub id: String,
    /// UUID of the market the order belongs to.
    #[serde(default)]
    pub market_id: Option<String>,
    /// The outcome being traded (`"YES"` or `"NO"`).
    #[serde(default)]
    pub outcome: Option<String>,
    /// `"BUY"` or `"SELL"`.
    pub side: String,
    /// Order type (`"LIMIT"` or `"MARKET"`).
    #[serde(default)]
    pub order_type: Option<String>,
    /// Self-trade prevention mode applied.
    #[serde(default)]
    pub stp_mode: Option<String>,
    /// Order status: `"open"`, `"filled"`, `"partial_filled"`, `"cancelled"`, `"expired"`, `"rejected"`.
    pub status: String,
    /// Requested amount in the specified currency.
    pub amount: f64,
    /// Limit price per share (0.00 for market orders).
    pub price: f64,
    /// Total size of the order in shares.
    #[serde(default)]
    pub size: Option<f64>,
    /// Number of shares filled so far.
    #[serde(default)]
    pub filled_size: Option<f64>,
    /// Number of shares remaining on the book.
    #[serde(default)]
    pub remaining_size: Option<f64>,
    /// Average fill price of fills so far.
    #[serde(default)]
    pub avg_fill_price: Option<f64>,
    /// Fee charged for the order.
    #[serde(default)]
    pub fee: Option<f64>,
    /// Number of shares received.
    #[serde(default)]
    pub quantity: Option<f64>,
    /// Currency used (`"USD"` or `"NGN"`).
    pub currency: String,
    /// UUID of the order owner.
    #[serde(default)]
    pub user_id: Option<String>,
    /// Whether the order is post-only.
    #[serde(default)]
    pub post_only: Option<bool>,
    /// ISO 8601 timestamp of creation.
    pub created_at: String,
    /// ISO 8601 timestamp of last update.
    pub updated_at: String,
}

/// Response from the list-orders endpoint.
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ListOrdersResponse {
    pub orders: Vec<Order>,
    pub pagination: Pagination,
}

/// Query parameters for the list-orders endpoint.
#[derive(Debug, Default, Clone)]
pub struct ListOrdersQuery {
    /// Filter by side: `"BUY"` or `"SELL"`.
    pub side: Option<String>,
    /// Filter by status: `"open"`, `"filled"`, `"partial_filled"`, etc.
    pub status: Option<String>,
    /// Filter by event UUID.
    pub event_id: Option<String>,
    /// Filter by market UUID.
    pub market_id: Option<String>,
    /// Filter by outcome UUID.
    pub outcome_id: Option<String>,
    /// Filter by currency: `"USD"` or `"NGN"`.
    pub currency: Option<String>,
    /// Page number (default: 1).
    pub page: Option<u32>,
    /// Results per page (default: 20).
    pub size: Option<u32>,
}

impl ListOrdersQuery {
    /// Build a URL-encoded query string from the configured filters.
    ///
    /// Only fields set to `Some(...)` are included in the output.
    /// Fields left as `None` are omitted entirely.
    pub fn to_query_string(&self) -> String {
        let mut params = BTreeMap::new();
        if let Some(ref v) = self.side {
            params.insert("side".into(), v.clone());
        }
        if let Some(ref v) = self.status {
            params.insert("status".into(), v.clone());
        }
        if let Some(ref v) = self.event_id {
            params.insert("eventId".into(), v.clone());
        }
        if let Some(ref v) = self.market_id {
            params.insert("marketId".into(), v.clone());
        }
        if let Some(ref v) = self.outcome_id {
            params.insert("outcomeId".into(), v.clone());
        }
        if let Some(ref v) = self.currency {
            params.insert("currency".into(), v.clone());
        }
        if let Some(p) = self.page {
            params.insert("page".into(), p.to_string());
        }
        if let Some(s) = self.size {
            params.insert("size".into(), s.to_string());
        }
        build_request(&params)
    }
}

// --------------------------------------------------------------------------
// Trades (market data) types
// --------------------------------------------------------------------------

/// Query parameters for the trades (market data) endpoint.
///
/// All filters are optional; omit them to include all markets.
#[derive(Debug, Default, Clone)]
pub struct TradesQuery {
    /// Filter by market UUID.
    pub market_id: Option<String>,
    /// Filter to a specific trade UUID.
    pub trade_id: Option<String>,
    /// Filter by order UUID. Matches trades where the order is on
    /// either the taker or maker side.
    pub order_id: Option<String>,
    /// Filter by outcome UUID. Matches trades on either side.
    pub outcome_id: Option<String>,
    /// Filter by user UUID. Matches trades where the user was either
    /// the taker or the maker.
    pub user_id: Option<String>,
    /// Only return trades created at or after this RFC3339 timestamp
    /// (e.g. `2026-02-17T00:00:00Z`).
    pub from_date: Option<String>,
    /// Only return trades created at or before this RFC3339 timestamp.
    pub to_date: Option<String>,
    /// Page number (1-indexed).
    pub page: Option<u32>,
    /// Number of trades per page (max 100).
    pub size: Option<u32>,
}

impl TradesQuery {
    /// Build a URL-encoded query string from the configured filters.
    ///
    /// Only fields set to `Some(...)` are included in the output.
    pub fn to_query_string(&self) -> String {
        let mut params = BTreeMap::new();
        if let Some(ref v) = self.market_id {
            params.insert("marketId".into(), v.clone());
        }
        if let Some(ref v) = self.trade_id {
            params.insert("id".into(), v.clone());
        }
        if let Some(ref v) = self.order_id {
            params.insert("orderId".into(), v.clone());
        }
        if let Some(ref v) = self.outcome_id {
            params.insert("outcomeId".into(), v.clone());
        }
        if let Some(ref v) = self.user_id {
            params.insert("userId".into(), v.clone());
        }
        if let Some(ref v) = self.from_date {
            params.insert("fromDate".into(), v.clone());
        }
        if let Some(ref v) = self.to_date {
            params.insert("toDate".into(), v.clone());
        }
        if let Some(p) = self.page {
            params.insert("page".into(), p.to_string());
        }
        if let Some(s) = self.size {
            params.insert("size".into(), s.to_string());
        }
        build_request(&params)
    }
}

// --------------------------------------------------------------------------
// PnL types
// --------------------------------------------------------------------------

/// Query parameters for the get-pnl endpoint.
#[derive(Debug, Default, Clone)]
pub struct GetPnLQuery {
    /// Predefined time window: `"12H"`, `"24H"`, `"1W"`, `"1M"`, `"1Y"`,
    /// `"THIS_WEEK"`, `"THIS_MONTH"`, `"THIS_YEAR"`.
    pub time_period: Option<String>,
    /// Custom start (ISO 8601). Must be paired with `end`.
    pub start: Option<String>,
    /// Custom end (ISO 8601). Must be paired with `start`.
    pub end: Option<String>,
    /// Currency filter: `"USD"` or `"NGN"`.
    pub currency: Option<String>,
    /// Include per-event breakdown. Defaults to `false`.
    pub breakdown: Option<bool>,
}

impl GetPnLQuery {
    /// Build a URL-encoded query string from the configured parameters.
    ///
    /// Only fields set to `Some(...)` are included in the output.
    pub fn to_query_string(&self) -> String {
        let mut params = BTreeMap::new();
        if let Some(ref v) = self.time_period {
            params.insert("timePeriod".into(), v.clone());
        }
        if let Some(ref v) = self.start {
            params.insert("start".into(), v.clone());
        }
        if let Some(ref v) = self.end {
            params.insert("end".into(), v.clone());
        }
        if let Some(ref v) = self.currency {
            params.insert("currency".into(), v.clone());
        }
        if let Some(b) = self.breakdown {
            params.insert("breakdown".into(), b.to_string());
        }
        build_request(&params)
    }
}

/// Per-event PnL breakdown item.
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct PnLBreakdownItem {
    /// UUID of the event.
    pub event_id: String,
    /// Human-readable title of the event.
    pub event_title: String,
    /// Aggregated realized PnL for this event.
    pub realized_pnl: f64,
    /// Currency the PnL is denominated in.
    pub currency: String,
    /// ISO 8601 timestamp of the most recent settlement or sell in this event.
    pub last_activity: String,
}

/// Response from the get-pnl endpoint.
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct GetPnLResponse {
    /// Total realized PnL (sum of settlement PnL and trade PnL).
    pub realized_pnl: f64,
    /// Realized PnL as a percentage of total cost basis.
    pub realized_pnl_percent: f64,
    /// PnL from resolved markets (payouts minus cost basis).
    pub settlement_pnl: f64,
    /// PnL from selling shares before market resolution.
    pub trade_pnl: f64,
    /// Number of settled positions that received a payout.
    pub wins: u64,
    /// Number of settled positions that received zero payout.
    pub losses: u64,
    /// Currency the PnL is denominated in.
    pub currency: String,
    /// Per-event PnL breakdown. Present when `breakdown=true` was requested.
    #[serde(default)]
    pub breakdown: Option<Vec<PnLBreakdownItem>>,
}

// --------------------------------------------------------------------------
// Portfolio types
// --------------------------------------------------------------------------

/// A summary of the parent event within a portfolio position.
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct PortfolioEvent {
    /// UUID of the event.
    pub id: String,
    /// Title of the event.
    pub title: String,
    /// Type of the event (`"single"` or `"combined"`).
    #[serde(rename = "type")]
    pub event_type: String,
    /// Trading engine used (`"AMM"` or `"CLOB"`).
    pub engine: String,
}

/// A summary of the market within a portfolio position.
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct PortfolioMarket {
    /// UUID of the market.
    pub id: String,
    /// Title of the market.
    pub title: String,
    /// Summary of the parent event.
    pub event: PortfolioEvent,
}

/// A single position in the portfolio.
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct OutcomeBalance {
    /// UUID of the position.
    pub id: String,
    /// The outcome held (`"YES"` or `"NO"`).
    pub outcome: String,
    /// UUID of the outcome.
    pub outcome_id: String,
    /// Total shares held.
    pub balance: f64,
    /// Shares available to sell.
    pub available_balance: f64,
    /// Average price paid per share.
    pub average_price: f64,
    /// Total amount invested.
    pub cost: f64,
    /// Current market value of the position.
    pub current_value: f64,
    /// Current price per share if sold now.
    pub sell_price: f64,
    /// Payout if this outcome resolves as the winner.
    pub payout_if_outcome_wins: f64,
    /// Percentage gain or loss from average price.
    pub percentage_change: f64,
    /// Currency this position is denominated in.
    pub currency: String,
    /// Summary of the market this position is in.
    pub market: PortfolioMarket,
    /// ISO 8601 timestamp of creation.
    pub created_at: String,
    /// ISO 8601 timestamp of last update.
    pub updated_at: String,
}

/// Response from the get-portfolio endpoint.
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct GetPortfolioResponse {
    /// List of positions across all markets.
    pub outcome_balances: Vec<OutcomeBalance>,
    /// Total amount invested across all positions.
    pub portfolio_cost: f64,
    /// Total current value of all positions.
    pub portfolio_current_value: f64,
    /// Overall portfolio gain or loss percentage.
    pub portfolio_percentage_change: f64,
    /// Pagination metadata.
    pub pagination: Pagination,
}

// --------------------------------------------------------------------------
// Cancel order types
// --------------------------------------------------------------------------

/// Response from the cancel-order endpoint.
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct CancelOrderResponse {
    /// Confirmation message (e.g. `"Order cancelled"`).
    pub message: String,
}

/// Request body for the batch-cancel-orders endpoint.
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct BatchCancelOrdersRequest {
    /// 1–100 order UUIDs to cancel.
    pub order_ids: Vec<String>,
}

/// Per-order outcome in a batch-cancel response.
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct BatchCancelResult {
    /// The order UUID submitted in the request.
    pub order_id: String,
    /// `true` if the cancel was accepted.
    pub success: bool,
    /// Error details. Present when `success` is `false`.
    #[serde(default)]
    pub error: Option<BatchOrderError>,
}

/// Response from the batch-cancel-orders endpoint.
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct BatchCancelOrdersResponse {
    /// Always `"CLOB"` for batch endpoints.
    pub engine: String,
    /// Per-order outcomes in the same order as the request.
    pub results: Vec<BatchCancelResult>,
    /// Aggregate counts.
    pub summary: BatchSummary,
}

// --------------------------------------------------------------------------
// Batch amend order types
// --------------------------------------------------------------------------

/// A single amend item in a batch-amend request.
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct BatchAmendOrdersItem {
    /// UUID of the order to amend. Must be owned by the caller.
    pub order_id: String,
    /// New limit price per share (0.01–0.99). Absolute, not a delta.
    /// Omit to keep the order's current price.
    /// At least one of `newPrice` or `newSize` must be supplied.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub new_price: Option<f64>,
    /// New TOTAL size of the order in the order's original currency.
    /// Must be greater than the order's `filledSize`.
    /// Omit to keep the order's current size.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub new_size: Option<f64>,
}

/// Request body for the batch-amend-orders endpoint.
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct BatchAmendOrdersRequest {
    /// 1–20 amend items. Each is processed independently.
    pub items: Vec<BatchAmendOrdersItem>,
}

/// Per-item outcome in a batch-amend response.
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct BatchAmendResult {
    /// Position of this item in the request `items` array (zero-based).
    pub index: u32,
    /// The order UUID submitted in the request.
    pub order_id: String,
    /// `true` if the amend transitioned the order to the new `(price, size)`.
    pub success: bool,
    /// The amended CLOB order. Present when `success` is `true`.
    #[serde(default)]
    pub order: Option<Order>,
    /// Error details. Present when `success` is `false`.
    #[serde(default)]
    pub error: Option<BatchOrderError>,
}

/// Response from the batch-amend-orders endpoint.
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct BatchAmendOrdersResponse {
    /// Always `"CLOB"` for batch endpoints.
    pub engine: String,
    /// Per-item outcomes in the same order as the request.
    pub results: Vec<BatchAmendResult>,
    /// Aggregate counts.
    pub summary: BatchSummary,
}

// --------------------------------------------------------------------------
// Mint / Burn types
// --------------------------------------------------------------------------

/// Request body for mint-shares or burn-shares.
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct MintBurnRequest {
    /// Amount to mint or burn in the selected currency.
    pub quantity: f64,
    /// `"USD"` (default) or `"NGN"`.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub currency: Option<String>,
}

/// Response from the mint-shares endpoint.
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct MintResponse {
    /// UUID of the mint operation.
    pub operation_id: String,
    /// UUID of the market.
    pub market_id: String,
    /// Normalized share quantity minted.
    pub quantity: f64,
    /// Current price of outcome 1 (YES).
    pub outcome1_price: f64,
    /// Current price of outcome 2 (NO).
    pub outcome2_price: f64,
}

/// Response from the burn-shares endpoint.
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct BurnResponse {
    /// UUID of the burn operation.
    pub operation_id: String,
    /// UUID of the market.
    pub market_id: String,
    /// Normalized share quantity burned.
    pub quantity: f64,
    /// Current price of outcome 1 (YES).
    pub outcome1_price: f64,
    /// Current price of outcome 2 (NO).
    pub outcome2_price: f64,
    /// Funds returned from the burn operation.
    pub proceeds: f64,
}

// --------------------------------------------------------------------------
// Quote types
// --------------------------------------------------------------------------

/// Response from the get-quote endpoint.
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct GetQuoteResponse {
    /// Average price per share (0.00–1.00).
    pub price: f64,
    /// Current market price before this trade executes.
    pub current_market_price: f64,
    /// Number of shares you will receive.
    pub quantity: f64,
    /// Total amount spent (including fee).
    pub amount: f64,
    /// Cost of shares before fee.
    pub cost_of_shares: f64,
    /// Trading fee charged.
    pub fee: f64,
    /// How much this trade moves the market price.
    pub price_impact_absolute: f64,
    /// Estimated profit percentage if the outcome wins.
    pub profit_percentage: f64,
    /// Multiplier for currency conversion (1 for USD, 100 for NGN).
    pub currency_base_multiplier: u64,
    /// Whether the full amount can be filled at the quoted price.
    pub complete_fill: bool,
    /// Whether this trade exceeds maximum liability limits.
    pub trade_goes_over_max_liability: bool,
}
