use bayse::{Bayse, MarketDataManager};

const API_KEY: &str = "";
const SECRET_KEY: &str = "";

#[tokio::test]
async fn test_get_price_history() {
    let client: MarketDataManager =
        Bayse::new(Some(API_KEY.to_string()), Some(SECRET_KEY.to_string()));
    let event_id = "";
    match client.get_price_history(event_id, None, None, None).await {
        Ok(_) => {}
        Err(e) => panic!("Error: {}", e),
    }
}

#[tokio::test]
async fn test_get_order_book() {
    let client: MarketDataManager =
        Bayse::new(Some(API_KEY.to_string()), Some(SECRET_KEY.to_string()));
    let event_id = Vec::new();
    match client.get_order_book(&event_id, None).await {
        Ok(_) => {}
        Err(e) => panic!("Error: {}", e),
    }
}

#[tokio::test]
async fn test_get_ticker() {
    let client: MarketDataManager =
        Bayse::new(Some(API_KEY.to_string()), Some(SECRET_KEY.to_string()));
    let event_id = "";
    match client.get_ticker(event_id).await {
        Ok(_) => {}
        Err(e) => panic!("Error: {}", e),
    }
}

#[tokio::test]
async fn test_get_trades() {
    let client: MarketDataManager =
        Bayse::new(Some(API_KEY.to_string()), Some(SECRET_KEY.to_string()));
    let event_ids = Vec::new();
    match client.get_trades(Some(&event_ids), None).await {
        Ok(_) => {}
        Err(e) => panic!("Error: {}", e),
    }
}
