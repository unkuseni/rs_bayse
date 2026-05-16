//! Market maker endpoints: liquidity rewards and maker rebates.

use crate::prelude::*;

/// Manager for market maker-related API endpoints.
///
/// Provides methods to query liquidity rewards and maker rebates for
/// the authenticated user.
#[derive(Clone)]
pub struct MarketMakerManager {
    /// The underlying HTTP client used for all requests.
    pub client: Client,
}

impl Bayse for MarketMakerManager {
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

impl MarketMakerManager {
    /// Get paginated list of the authenticated user's liquidity reward payouts.
    ///
    /// **Auth level:** Read (requires `X-Public-Key` header).
    ///
    /// # Parameters
    ///
    /// * `page` – Optional page number (1-based). Defaults to 1.
    /// * `size` – Optional number of records per page. Defaults to the
    ///   server's default page size.
    ///
    /// # Returns
    ///
    /// A [`LiquidityRewardsResponse`] containing an array of
    /// [`LiquidityRewardRecord`] values and pagination metadata.
    pub async fn get_liquidity_rewards(
        &self,
        page: Option<u32>,
        size: Option<u32>,
    ) -> Result<LiquidityRewardsResponse, BayseError> {
        let mut params = BTreeMap::new();
        if let Some(p) = page {
            params.insert("page".to_string(), p.to_string());
        }
        if let Some(s) = size {
            params.insert("size".to_string(), s.to_string());
        }
        let qs = build_request(&params);
        self.client
            .get_read(
                API::MarketMaker(MarketMakerEndpoint::LiquidityRewards).as_ref(),
                Some(qs),
            )
            .await
    }

    /// Get the authenticated user's in-progress reward accumulation across active epochs.
    ///
    /// **Auth level:** Read (requires `X-Public-Key` header).
    ///
    /// # Returns
    ///
    /// An [`ActiveLiquidityRewardsResponse`] containing an array of
    /// [`ActiveLiquidityReward`] values with estimated payouts for
    /// the current epoch window.
    pub async fn get_active_liquidity_rewards(
        &self,
    ) -> Result<ActiveLiquidityRewardsResponse, BayseError> {
        self.client
            .get_read(
                API::MarketMaker(MarketMakerEndpoint::ActiveLiquidityRewards).as_ref(),
                None,
            )
            .await
    }

    /// Get paginated list of the authenticated user's maker rebate payouts.
    ///
    /// **Auth level:** Read (requires `X-Public-Key` header).
    ///
    /// # Parameters
    ///
    /// * `page` – Optional page number (1-based). Defaults to 1.
    /// * `size` – Optional number of records per page. Defaults to the
    ///   server's default page size.
    ///
    /// # Returns
    ///
    /// A [`MakerRebatesResponse`] containing an array of
    /// [`MakerRebateRecord`] values and pagination metadata.
    pub async fn get_maker_rebates(
        &self,
        page: Option<u32>,
        size: Option<u32>,
    ) -> Result<MakerRebatesResponse, BayseError> {
        let mut params = BTreeMap::new();
        if let Some(p) = page {
            params.insert("page".to_string(), p.to_string());
        }
        if let Some(s) = size {
            params.insert("size".to_string(), s.to_string());
        }
        let qs = build_request(&params);
        self.client
            .get_read(
                API::MarketMaker(MarketMakerEndpoint::MakerRebates).as_ref(),
                Some(qs),
            )
            .await
    }

    /// Get the authenticated user's in-progress maker rebate accumulation across active epochs.
    ///
    /// **Auth level:** Read (requires `X-Public-Key` header).
    ///
    /// # Returns
    ///
    /// An [`ActiveMakerRebatesResponse`] containing an array of
    /// [`ActiveMakerRebate`] values with estimated rebates for
    /// the current epoch window.
    pub async fn get_active_maker_rebates(&self) -> Result<ActiveMakerRebatesResponse, BayseError> {
        self.client
            .get_read(
                API::MarketMaker(MarketMakerEndpoint::ActiveMakerRebates).as_ref(),
                None,
            )
            .await
    }
}
