use bayse::{Bayse, MarketDataManager, TradesQuery};

// NOTE: These are live integration tests. They hit the production Bayse API
// (https://relay.bayse.markets) and require real event/market/outcome IDs.
// They are ignored by default so `cargo test` stays hermetic; run them with:
//     cargo test --test market_data_tests -- --ignored
const API_KEY: &str = "";
const SECRET_KEY: &str = "";

#[tokio::test]
#[ignore]
async fn test_get_price_history() {
    let client: MarketDataManager =
        Bayse::new(Some(API_KEY.to_string()), Some(SECRET_KEY.to_string()));
    let event_id = "";
    match client
        .get_price_history(event_id, Some("24H"), None, None)
        .await
    {
        Ok(_) => {}
        Err(e) => panic!("Error: {}", e),
    }
}

#[tokio::test]
#[ignore]
async fn test_get_order_book() {
    let client: MarketDataManager =
        Bayse::new(Some(API_KEY.to_string()), Some(SECRET_KEY.to_string()));
    let outcome_ids: Vec<&str> = Vec::new();
    match client.get_order_book(&outcome_ids, None, None).await {
        Ok(_) => {}
        Err(e) => panic!("Error: {}", e),
    }
}

#[tokio::test]
#[ignore]
async fn test_get_ticker() {
    let client: MarketDataManager =
        Bayse::new(Some(API_KEY.to_string()), Some(SECRET_KEY.to_string()));
    let market_id = "";
    match client.get_ticker(market_id, Some("YES"), None).await {
        Ok(_) => {}
        Err(e) => panic!("Error: {}", e),
    }
}

#[tokio::test]
#[ignore]
async fn test_get_trades() {
    let client: MarketDataManager =
        Bayse::new(Some(API_KEY.to_string()), Some(SECRET_KEY.to_string()));
    let query = TradesQuery {
        page: Some(1),
        size: Some(10),
        ..Default::default()
    };
    match client.get_trades(&query).await {
        Ok(_) => {}
        Err(e) => panic!("Error: {}", e),
    }
}
