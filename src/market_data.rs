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
    /// * `time_period` – Optional time window: `12H`, `24H`, `1W`, `1M`,
    ///   or `1Y`. The server defaults to `24H` if omitted.
    /// * `market_ids` – Optional filter to specific market UUIDs
    ///   (comma-separated in the request). Omit to return all markets
    ///   in the event.
    /// * `outcome` – Optional filter to a specific outcome: `YES` or `NO`.
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
    ///         Some("1W"),
    ///         None,
    ///         Some("YES"),
    ///     ).await?;
    ///     println!("{history:#?}");
    ///     Ok(())
    /// }
    /// ```
    pub async fn get_price_history(
        &self,
        event_id: &str,
        time_period: Option<&str>,
        market_ids: Option<&[&str]>,
        outcome: Option<&str>,
    ) -> Result<serde_json::Value, BayseError> {
        let mut params = BTreeMap::new();
        if let Some(tp) = time_period {
            params.insert("timePeriod".to_string(), tp.to_string());
        }
        if let Some(ids) = market_ids {
            if !ids.is_empty() {
                params.insert("marketId[]".to_string(), ids.join(","));
            }
        }
        if let Some(o) = outcome {
            params.insert("outcome".to_string(), o.to_string());
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
    /// * `outcome_ids` – One or more outcome UUIDs to query
    ///   (sent as `outcomeId[]`, comma-separated in the request).
    /// * `depth` – Optional maximum number of price levels on each side.
    ///   The server defaults to 10 if omitted.
    /// * `currency` – Optional currency for price display: `USD` or `NGN`.
    ///
    /// # Returns
    ///
    /// A JSON object containing bid and ask arrays for each requested
    /// market, with each level showing `price`, `size`, and `order_count`.
    pub async fn get_order_book(
        &self,
        outcome_ids: &[&str],
        depth: Option<u32>,
        currency: Option<&str>,
    ) -> Result<serde_json::Value, BayseError> {
        let mut params = BTreeMap::new();
        params.insert("outcomeId[]".to_string(), outcome_ids.join(","));
        if let Some(d) = depth {
            params.insert("depth".to_string(), d.to_string());
        }
        if let Some(c) = currency {
            params.insert("currency".to_string(), c.to_string());
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
    /// * `outcome` – The outcome label: `YES` or `NO`. Required if
    ///   `outcome_id` is not provided.
    /// * `outcome_id` – UUID of the outcome. Required if `outcome` is
    ///   not provided.
    ///
    /// # Returns
    ///
    /// A JSON object with `price`, `change`, `volume`, `high`, `low`,
    /// and `timestamp` fields for the given market.
    pub async fn get_ticker(
        &self,
        market_id: &str,
        outcome: Option<&str>,
        outcome_id: Option<&str>,
    ) -> Result<serde_json::Value, BayseError> {
        let mut params = BTreeMap::new();
        if let Some(o) = outcome {
            params.insert("outcome".to_string(), o.to_string());
        }
        if let Some(oid) = outcome_id {
            params.insert("outcomeId".to_string(), oid.to_string());
        }
        let qs = build_request(&params);
        let endpoint = format!("/v1/pm/markets/{market_id}/ticker");
        self.client.get(&endpoint, Some(qs)).await
    }

    /// Get recent executed trades (CLOB markets only).
    ///
    /// **Auth level:** None (public endpoint).
    ///
    /// # Parameters
    ///
    /// * `query` – Filters for the trades list: `market_id`, `trade_id`,
    ///   `order_id`, `outcome_id`, `user_id`, `from_date`, `to_date`,
    ///   `page`, and `size`. Omit filters to include all markets.
    ///
    /// # Returns
    ///
    /// A JSON object containing an array of recent trades, each with
    /// `market_id`, `side`, `price`, `size`, `timestamp`, and `trade_id`.
    pub async fn get_trades(
        &self,
        query: &TradesQuery,
    ) -> Result<serde_json::Value, BayseError> {
        let qs = query.to_query_string();
        self.client
            .get(
                API::MarketData(MarketDataEndpoint::Trades).as_ref(),
                Some(qs),
            )
            .await
    }
}
