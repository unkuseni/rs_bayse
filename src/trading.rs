//! Trading endpoints for prediction markets.
//!
//! Covers event browsing, quoting, order placement, portfolio, PnL, and
//! mint/burn share operations.

use crate::prelude::*;

/// Manager for prediction market trading endpoints.
#[derive(Clone)]
pub struct TradingManager {
    pub client: Client,
}

impl Bayse for TradingManager {
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

impl TradingManager {
    // ------------------------------------------------------------------
    // Events
    // ------------------------------------------------------------------

    /// Get a paginated list of prediction market events.
    ///
    /// Uses `page` and `size` query parameters. Requires a valid API key (read-level).
    pub async fn list_events(
        &self,
        page: Option<u32>,
        size: Option<u32>,
    ) -> Result<ListEventsResponse, BayseError> {
        let mut params = BTreeMap::new();
        if let Some(p) = page {
            params.insert("page".to_string(), p.to_string());
        }
        if let Some(s) = size {
            params.insert("size".to_string(), s.to_string());
        }
        let qs = build_request(&params);
        self.client
            .get_read(API::Trading(TradingEndpoint::ListEvents).as_ref(), Some(qs))
            .await
    }

    /// Get a specific prediction market event by ID.
    ///
    /// Returns the full [`Event`] including all markets, outcomes, and
    /// metadata for the given event UUID.
    ///
    /// **Auth level:** Read (requires `X-Public-Key` header).
    ///
    /// # Parameters
    ///
    /// * `event_id` – The UUID of the event to retrieve.
    pub async fn get_event(&self, event_id: &str) -> Result<Event, BayseError> {
        let endpoint = format!("/v1/pm/events/{event_id}");
        self.client.get_read(&endpoint, None).await
    }

    /// Get a specific prediction market event by its slug.
    ///
    /// Slugs are URL-friendly identifiers (e.g.
    /// `"who-will-win-the-2027-presidential-election"`). This is a
    /// convenient alternative to looking up an event by UUID.
    ///
    /// **Auth level:** Read (requires `X-Public-Key` header).
    ///
    /// # Parameters
    ///
    /// * `slug` – The URL slug of the event.
    pub async fn get_event_by_slug(&self, slug: &str) -> Result<Event, BayseError> {
        let endpoint = format!("/v1/pm/events/slug/{slug}");
        self.client.get_read(&endpoint, None).await
    }

    /// Get a paginated list of event series.
    ///
    /// **Auth level:** Read (requires `X-Public-Key` header).
    ///
    /// # Parameters
    ///
    /// * `page` – Optional page number (1-based). Defaults to 1.
    /// * `size` – Optional number of series per page.
    pub async fn list_series(
        &self,
        page: Option<u32>,
        size: Option<u32>,
    ) -> Result<ListSeriesResponse, BayseError> {
        let mut params = BTreeMap::new();
        if let Some(p) = page {
            params.insert("page".to_string(), p.to_string());
        }
        if let Some(s) = size {
            params.insert("size".to_string(), s.to_string());
        }
        let qs = build_request(&params);
        self.client
            .get_read(API::Trading(TradingEndpoint::ListSeries).as_ref(), Some(qs))
            .await
    }

    /// Get a lightweight list of events belonging to a series.
    ///
    /// Returns a compact view of events in a series — useful for listing
    /// rounds without the full event detail. The response shape differs
    /// from the full event object returned by [`get_event`](Self::get_event).
    ///
    /// **Auth level:** Read (requires `X-Public-Key` header).
    ///
    /// # Parameters
    ///
    /// * `series_slug` – The URL slug of the series (e.g.
    ///   `"crypto-btc-15min"`).
    /// * `page` – Optional page number (1-based). Defaults to 1.
    /// * `size` – Optional number of events per page.
    pub async fn get_series_events(
        &self,
        series_slug: &str,
        page: Option<u32>,
        size: Option<u32>,
    ) -> Result<Vec<SeriesEventSummary>, BayseError> {
        let mut params = BTreeMap::new();
        if let Some(p) = page {
            params.insert("page".to_string(), p.to_string());
        }
        if let Some(s) = size {
            params.insert("size".to_string(), s.to_string());
        }
        let qs = build_request(&params);
        let endpoint = format!("/v1/pm/events/series/{series_slug}/lean-events");
        self.client.get_read(&endpoint, Some(qs)).await
    }

    // ------------------------------------------------------------------
    // Quotes & Orders
    // ------------------------------------------------------------------

    /// Get a price quote before placing an order.
    ///
    /// This endpoint is **public** — no authentication is required.
    /// If the client has an API key configured, the `X-Public-Key` header is
    /// included so the quote includes personalised profit estimates based on
    /// your existing position.
    ///
    /// Always get a quote before placing an order to confirm the expected cost,
    /// shares, and fees. The quoted price is indicative — the actual fill price
    /// may differ slightly in fast-moving markets.
    pub async fn get_quote(
        &self,
        event_id: &str,
        market_id: &str,
        req: &GetQuoteRequest,
    ) -> Result<GetQuoteResponse, BayseError> {
        let endpoint = format!("/v1/pm/events/{event_id}/markets/{market_id}/quote");
        let url = format!("{}{}", self.client.host, endpoint);
        let body_str = serde_json::to_string(req)?;

        let mut request = self
            .client
            .inner()
            .post(&url)
            .header(USER_AGENT, "rs_bayse/0.1.0")
            .header(CONTENT_TYPE, "application/json")
            .body(body_str);

        // Pass the public key for personalised profit estimates if available.
        if let Some(ref key) = self.client.api_key {
            request = request.header("X-Public-Key", key);
        }

        self.client.handler(request).await
    }

    /// Place a buy or sell order on a prediction market.
    ///
    /// Requires write-level authentication (HMAC-SHA256).
    ///
    /// Always get a quote first via [`get_quote`](Self::get_quote) to confirm
    /// the expected cost, shares, and fees before placing an order.
    pub async fn place_order(
        &self,
        event_id: &str,
        market_id: &str,
        req: &PlaceOrderRequest,
    ) -> Result<PlaceOrderResponse, BayseError> {
        let endpoint = format!("/v1/pm/events/{event_id}/markets/{market_id}/orders");
        let body_str = serde_json::to_string(req)?;
        self.client.post_signed(&endpoint, Some(body_str)).await
    }

    /// Place up to 20 CLOB orders across one or more markets in a single round-trip.
    ///
    /// Orders may span multiple markets and events — each item carries only
    /// `outcome_id`, and the server resolves the parent market and event.
    ///
    /// Requires write-level authentication (HMAC-SHA256).
    pub async fn batch_place_orders(
        &self,
        req: &BatchPlaceOrdersRequest,
    ) -> Result<BatchPlaceOrdersResponse, BayseError> {
        let body_str = serde_json::to_string(req)?;
        self.client
            .post_signed(
                API::Trading(TradingEndpoint::BatchPlaceOrders).as_ref(),
                Some(body_str),
            )
            .await
    }

    // ------------------------------------------------------------------
    // Portfolio & PnL
    // ------------------------------------------------------------------

    /// Get your current positions across all markets.
    ///
    /// Requires a valid API key (read-level).
    pub async fn get_portfolio(&self) -> Result<GetPortfolioResponse, BayseError> {
        self.client
            .get_read(API::Trading(TradingEndpoint::GetPortfolio).as_ref(), None)
            .await
    }

    /// Get your realised profit and loss over a time period.
    ///
    /// Supports predefined rolling/calendar windows (`GetPnLQuery::time_period`)
    /// or custom start/end ISO 8601 timestamps.
    ///
    /// Requires a valid API key (read-level).
    pub async fn get_pnl(&self, query: &GetPnLQuery) -> Result<GetPnLResponse, BayseError> {
        let qs = query.to_query_string();
        self.client
            .get_read(API::Trading(TradingEndpoint::GetPnL).as_ref(), Some(qs))
            .await
    }

    // ------------------------------------------------------------------
    // Orders
    // ------------------------------------------------------------------

    /// Get a paginated list of your orders.
    ///
    /// Supports filtering by side, status, event, market, outcome, and
    /// currency via `ListOrdersQuery`.
    ///
    /// Requires a valid API key (read-level).
    pub async fn list_orders(
        &self,
        query: &ListOrdersQuery,
    ) -> Result<ListOrdersResponse, BayseError> {
        let qs = query.to_query_string();
        self.client
            .get_read(API::Trading(TradingEndpoint::ListOrders).as_ref(), Some(qs))
            .await
    }

    /// Get details of a specific order.
    ///
    /// Requires a valid API key (read-level).
    pub async fn get_order(&self, order_id: &str) -> Result<Order, BayseError> {
        let endpoint = format!("/v1/pm/orders/{order_id}");
        self.client.get_read(&endpoint, None).await
    }

    /// Cancel an open or partially filled CLOB order.
    ///
    /// Only CLOB orders with status `open` or `partial_filled` can be
    /// cancelled. AMM orders execute instantly and cannot be cancelled —
    /// sell your shares to exit instead.
    ///
    /// Requires write-level authentication (HMAC-SHA256).
    pub async fn cancel_order(&self, order_id: &str) -> Result<CancelOrderResponse, BayseError> {
        let endpoint = format!("/v1/pm/orders/{order_id}");
        self.client.delete_signed(&endpoint, None).await
    }

    /// Cancel up to 100 CLOB orders in a single round-trip.
    ///
    /// Order IDs may belong to different markets and events. AMM orders are
    /// rejected per-item with `UNSUPPORTED_ENGINE`. Ownership is enforced
    /// upstream — order IDs the caller does not own return `ORDER_NOT_FOUND`.
    ///
    /// Requires write-level authentication (HMAC-SHA256).
    pub async fn batch_cancel_orders(
        &self,
        req: &BatchCancelOrdersRequest,
    ) -> Result<BatchCancelOrdersResponse, BayseError> {
        let body_str = serde_json::to_string(req)?;
        self.client
            .post_signed(
                API::Trading(TradingEndpoint::BatchCancelOrders).as_ref(),
                Some(body_str),
            )
            .await
    }

    /// Amend the price and/or size of up to 20 open CLOB orders in a single
    /// round-trip.
    ///
    /// Each item names an existing `orderId` you own and supplies the new
    /// `price`, the new total `size`, or both. Orders may belong to different
    /// markets and events. CLOB-only: any AMM order is rejected per-item with
    /// `UNSUPPORTED_ENGINE`.
    ///
    /// Amend mutates an order in place — preserving time priority when
    /// possible — instead of cancelling and re-placing.
    ///
    /// Self-trade prevention on amend is a fixed server policy:
    /// **always `CANCEL_OLDEST`**. If the amend would put the order in a
    /// position that crosses a same-user resting order, the resting crosser
    /// is cancelled and the amend proceeds.
    ///
    /// Requires write-level authentication (HMAC-SHA256).
    pub async fn batch_amend_orders(
        &self,
        req: &BatchAmendOrdersRequest,
    ) -> Result<BatchAmendOrdersResponse, BayseError> {
        let body_str = serde_json::to_string(req)?;
        self.client
            .post_signed(
                API::Trading(TradingEndpoint::BatchAmendOrders).as_ref(),
                Some(body_str),
            )
            .await
    }

    // ------------------------------------------------------------------
    // Mint / Burn Shares
    // ------------------------------------------------------------------

    /// Deposit funds and receive equal YES and NO shares for a market.
    ///
    /// Minting creates new shares for a binary market. You deposit funds and
    /// receive an equal number of YES and NO shares. Minting does not affect
    /// market prices since it creates both sides equally.
    ///
    /// Requires write-level authentication (HMAC-SHA256).
    pub async fn mint_shares(
        &self,
        market_id: &str,
        req: &MintBurnRequest,
    ) -> Result<MintResponse, BayseError> {
        let endpoint = format!("/v1/pm/markets/{market_id}/mint");
        let body_str = serde_json::to_string(req)?;
        self.client.post_signed(&endpoint, Some(body_str)).await
    }

    /// Destroy equal YES and NO shares and receive funds back.
    ///
    /// Burning is the reverse of minting. You surrender an equal number of
    /// YES and NO shares and receive funds back. You must hold sufficient
    /// shares of both outcomes to burn.
    ///
    /// Requires write-level authentication (HMAC-SHA256).
    pub async fn burn_shares(
        &self,
        market_id: &str,
        req: &MintBurnRequest,
    ) -> Result<BurnResponse, BayseError> {
        let endpoint = format!("/v1/pm/markets/{market_id}/burn");
        let body_str = serde_json::to_string(req)?;
        self.client.post_signed(&endpoint, Some(body_str)).await
    }

    // ------------------------------------------------------------------
    // Activities
    // ------------------------------------------------------------------

    /// Get your trading activity history.
    ///
    /// Returns a paginated list of past trades, deposits, withdrawals, and
    /// other account activity as typed [`Activity`] records.
    ///
    /// **Auth level:** Read (requires `X-Public-Key` header).
    ///
    /// # Parameters
    ///
    /// * `activity_type` – Optional filter: `buys`, `sells`, `limits`, or
    ///   `payout`. When omitted, all activity types except trade fills
    ///   are returned.
    /// * `page` – Optional page number (1-based). Defaults to 1.
    /// * `size` – Optional number of activities per page.
    pub async fn get_activities(
        &self,
        activity_type: Option<&str>,
        page: Option<u32>,
        size: Option<u32>,
    ) -> Result<ActivitiesResponse, BayseError> {
        let mut params = BTreeMap::new();
        if let Some(t) = activity_type {
            params.insert("type".to_string(), t.to_string());
        }
        if let Some(p) = page {
            params.insert("page".to_string(), p.to_string());
        }
        if let Some(s) = size {
            params.insert("size".to_string(), s.to_string());
        }
        let qs = build_request(&params);
        self.client
            .get_read(API::Trading(TradingEndpoint::Activities).as_ref(), Some(qs))
            .await
    }

    // ------------------------------------------------------------------
    // Sports markets
    // ------------------------------------------------------------------

    /// Get a paginated list of sports games, optionally filtered by league
    /// or sport.
    ///
    /// **Auth level:** None (public endpoint).
    ///
    /// # Parameters
    ///
    /// * `league` – Optional league key (e.g. `"England - Premier League"`).
    /// * `sport` – Optional sport (e.g. `"soccer"`, `"basketball"`).
    /// * `page` – Optional page number (1-based). Defaults to 1.
    /// * `size` – Optional results per page (default 50, max 100).
    pub async fn list_sports_games(
        &self,
        league: Option<&str>,
        sport: Option<&str>,
        page: Option<u32>,
        size: Option<u32>,
    ) -> Result<SportsGamesResponse, BayseError> {
        let mut params = BTreeMap::new();
        if let Some(l) = league {
            params.insert("league".to_string(), l.to_string());
        }
        if let Some(s) = sport {
            params.insert("sport".to_string(), s.to_string());
        }
        if let Some(p) = page {
            params.insert("page".to_string(), p.to_string());
        }
        if let Some(sz) = size {
            params.insert("size".to_string(), sz.to_string());
        }
        let qs = build_request(&params);
        self.client.get("/v1/pm/sports/games", Some(qs)).await
    }

    /// Get a list of all supported sports leagues.
    ///
    /// **Auth level:** None (public endpoint).
    pub async fn list_sports_leagues(&self) -> Result<SportsLeaguesResponse, BayseError> {
        self.client.get("/v1/pm/sports/leagues", None).await
    }

    /// Get a paginated list of sports teams, optionally filtered by league
    /// or sport.
    ///
    /// **Auth level:** None (public endpoint).
    ///
    /// # Parameters
    ///
    /// * `league` – Optional league key (e.g. `"England - Premier League"`).
    /// * `sport` – Optional sport (e.g. `"soccer"`, `"basketball"`).
    /// * `page` – Optional page number (1-based). Defaults to 1.
    /// * `size` – Optional results per page (default 50, max 100).
    pub async fn list_sports_teams(
        &self,
        league: Option<&str>,
        sport: Option<&str>,
        page: Option<u32>,
        size: Option<u32>,
    ) -> Result<SportsTeamsResponse, BayseError> {
        let mut params = BTreeMap::new();
        if let Some(l) = league {
            params.insert("league".to_string(), l.to_string());
        }
        if let Some(s) = sport {
            params.insert("sport".to_string(), s.to_string());
        }
        if let Some(p) = page {
            params.insert("page".to_string(), p.to_string());
        }
        if let Some(sz) = size {
            params.insert("size".to_string(), sz.to_string());
        }
        let qs = build_request(&params);
        self.client.get("/v1/pm/sports/teams", Some(qs)).await
    }
}
