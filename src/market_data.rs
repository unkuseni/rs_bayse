//! Market data endpoints: price history, order book, ticker, and trades.

use crate::prelude::*;

/// Manager for market data (public) endpoints.
///
/// Provides access to price history, order book snapshots, real-time
/// tickers, and recent trades for prediction markets.
#[derive(Clone)]
pub struct MarketDataManager {
    /// The underlying HTTP client used for all requests.
    pub client: Client,
}

impl Bayse for MarketDataManager {
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

impl MarketDataManager {
    /// Get historical price data for a prediction market event.
    ///
    /// **Auth level:** None (public endpoint).
    ///
    /// # Parameters
    ///
    /// * `event_id` – The unique ID of the prediction market event.
    /// * `from` – Optional start timestamp (Unix milliseconds). If omitted
    ///   the API defaults to a reasonable look-back window.
    /// * `to` – Optional end timestamp (Unix milliseconds). Defaults to
    ///   the current time if omitted.
    /// * `resolution` – Optional candle width, e.g. `"1m"`, `"5m"`,
    ///   `"15m"`, `"1h"`, `"1d"`. The server determines the default
    ///   if not provided.
    ///
    /// # Returns
    ///
    /// A JSON object containing an array of price candles, each with
    /// `open`, `high`, `low`, `close`, `volume`, and `timestamp` fields.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use bayse::prelude::*;
    ///
    /// #[tokio::main]
    /// async fn main() -> Result<(), BayseError> {
    ///     let md = MarketDataManager::new(None, None);
    ///     let history = md.get_price_history(
    ///         "event_123",
    ///         None,
    ///         None,
    ///         Some("1h"),
    ///     ).await?;
    ///     println!("{history:#?}");
    ///     Ok(())
    /// }
    /// ```
    pub async fn get_price_history(
        &self,
        event_id: &str,
        from: Option<i64>,
        to: Option<i64>,
        resolution: Option<&str>,
    ) -> Result<serde_json::Value, BayseError> {
        let mut params = BTreeMap::new();
        if let Some(f) = from {
            params.insert("from".to_string(), f.to_string());
        }
        if let Some(t) = to {
            params.insert("to".to_string(), t.to_string());
        }
        if let Some(r) = resolution {
            params.insert("resolution".to_string(), r.to_string());
        }
        let qs = build_request(&params);
        let endpoint = format!("/v1/pm/events/{event_id}/price-history");
        self.client.get(&endpoint, Some(qs)).await
    }

    /// Get the live order book for one or more outcomes (CLOB markets only).
    ///
    /// **Auth level:** None (public endpoint).
    ///
    /// # Parameters
    ///
    /// * `market_ids` – One or more market outcome IDs to query
    ///   (comma-separated in the request).
    /// * `depth` – Optional maximum number of price levels on each side.
    ///   If omitted the server returns its default depth.
    ///
    /// # Returns
    ///
    /// A JSON object containing bid and ask arrays for each requested
    /// market, with each level showing `price`, `size`, and `order_count`.
    pub async fn get_order_book(
        &self,
        market_ids: &[&str],
        depth: Option<u32>,
    ) -> Result<serde_json::Value, BayseError> {
        let mut params = BTreeMap::new();
        params.insert("marketIds".to_string(), market_ids.join(","));
        if let Some(d) = depth {
            params.insert("depth".to_string(), d.to_string());
        }
        let qs = build_request(&params);
        self.client
            .get(
                API::MarketData(MarketDataEndpoint::OrderBook).as_ref(),
                Some(qs),
            )
            .await
    }

    /// Get real-time price and volume statistics for a market outcome.
    ///
    /// **Auth level:** None (public endpoint).
    ///
    /// # Parameters
    ///
    /// * `market_id` – The market outcome ID to get the ticker for.
    ///
    /// # Returns
    ///
    /// A JSON object with `price`, `change`, `volume`, `high`, `low`,
    /// and `timestamp` fields for the given market.
    pub async fn get_ticker(&self, market_id: &str) -> Result<serde_json::Value, BayseError> {
        let endpoint = format!("/v1/pm/markets/{market_id}/ticker");
        self.client.get(&endpoint, None).await
    }

    /// Get recent executed trades (CLOB markets only).
    ///
    /// **Auth level:** None (public endpoint).
    ///
    /// # Parameters
    ///
    /// * `market_ids` – Optional filter to return trades only for the
    ///   given market outcome IDs. If omitted all markets are included.
    /// * `limit` – Optional maximum number of trades to return.
    ///
    /// # Returns
    ///
    /// A JSON array of recent trades, each with `market_id`, `side`,
    /// `price`, `size`, `timestamp`, and `trade_id`.
    pub async fn get_trades(
        &self,
        market_ids: Option<&[&str]>,
        limit: Option<u32>,
    ) -> Result<serde_json::Value, BayseError> {
        let mut params = BTreeMap::new();
        if let Some(ids) = market_ids {
            params.insert("marketIds".to_string(), ids.join(","));
        }
        if let Some(l) = limit {
            params.insert("limit".to_string(), l.to_string());
        }
        let qs = build_request(&params);
        self.client
            .get(
                API::MarketData(MarketDataEndpoint::Trades).as_ref(),
                Some(qs),
            )
            .await
    }
}
