//! Integration tests for the WebSocket module.
//!
//! Tests cover:
//! - WsEvent JSON deserialisation from sample payloads matching the API docs
//! - WsSubscription / WsAuth serialisation to the expected wire format
//! - WebSocketHandler trait closure integration
//! - Brief live connection to the realtime endpoint (when API key is set)

use bayse::prelude::*;

const WS_HOST: &str = "wss://socket.bayse.markets";

// ---------------------------------------------------------------------------
// WsEvent deserialisation
// ---------------------------------------------------------------------------

#[test]
fn test_parse_connected_event() {
    let json = r#"{
        "type": "connected",
        "status": "connected",
        "clientId": "a1b2c3d4-e5f6-7890-abcd-ef1234567890",
        "message": "Successfully connected to WebSocket server",
        "timestamp": 1700000000000
    }"#;

    let event = WsEvent::from_json(json).expect("should parse connected event");
    match event {
        WsEvent::Connected(e) => {
            assert_eq!(e._type, "connected");
            assert_eq!(e.status, "connected");
            assert_eq!(e.client_id, "a1b2c3d4-e5f6-7890-abcd-ef1234567890");
            assert_eq!(e.message, "Successfully connected to WebSocket server");
            assert_eq!(e.timestamp, 1_700_000_000_000);
        }
        other => panic!("Expected Connected, got {other:?}"),
    }
}

#[test]
fn test_parse_pong_event() {
    let json = r#"{
        "type": "pong",
        "timestamp": 1700000000000
    }"#;

    let event = WsEvent::from_json(json).expect("should parse pong event");
    match event {
        WsEvent::Pong(e) => {
            assert_eq!(e._type, "pong");
            assert_eq!(e.timestamp, 1_700_000_000_000);
        }
        other => panic!("Expected Pong, got {other:?}"),
    }
}

#[test]
fn test_parse_error_event() {
    let json = r#"{
        "type": "error",
        "data": {
            "message": "auth required: include {\"auth\":{\"accessToken\":\"...\"}} or {\"auth\":{\"apiKey\":\"...\"}} in every message"
        },
        "timestamp": 1700000000000
    }"#;

    let event = WsEvent::from_json(json).expect("should parse error event");
    match event {
        WsEvent::Error(e) => {
            assert_eq!(e._type, "error");
            assert!(e.data.message.contains("auth required"));
            assert_eq!(e.timestamp, 1_700_000_000_000);
        }
        other => panic!("Expected Error, got {other:?}"),
    }
}

#[test]
fn test_parse_subscribed_event() {
    let json = r#"{
        "type": "subscribed",
        "room": "prices:evt_123",
        "message": "Subscribed to: prices:evt_123",
        "timestamp": 1700000000000
    }"#;

    let event = WsEvent::from_json(json).expect("should parse subscribed event");
    match event {
        WsEvent::Subscribed(e) => {
            assert_eq!(e._type, "subscribed");
            assert_eq!(e.room, "prices:evt_123");
            assert_eq!(e.message.as_deref(), Some("Subscribed to: prices:evt_123"));
            assert_eq!(e.timestamp, Some(1_700_000_000_000));
        }
        other => panic!("Expected Subscribed, got {other:?}"),
    }
}

#[test]
fn test_parse_unsubscribed_event() {
    let json = r#"{
        "type": "unsubscribed",
        "room": "prices:evt_123",
        "message": "Unsubscribed from: prices:evt_123",
        "timestamp": 1700000000000
    }"#;

    let event = WsEvent::from_json(json).expect("should parse unsubscribed event");
    match event {
        WsEvent::Unsubscribed(e) => {
            assert_eq!(e._type, "unsubscribed");
            assert_eq!(e.room, "prices:evt_123");
        }
        other => panic!("Expected Unsubscribed, got {other:?}"),
    }
}

#[test]
fn test_parse_buy_order_event() {
    let json = r#"{
        "type": "buy_order",
        "data": {
            "user": { "id": "usr_8b5c3c3a", "tag": "prediction_pro", "imageUrl": "https://cdn.bayse.markets/users/8b5c3c3a.png" },
            "order": {
                "id": "ord_7f5e2a1c", "amount": 100.00, "quantity": 150.0, "price": 0.65,
                "status": "FILLED", "type": "BUY", "outcome": "YES", "outcomeLabel": "Yes",
                "currency": "USD", "createdAt": "2026-02-17T10:30:00Z", "updatedAt": "2026-02-17T10:30:01Z"
            },
            "event": {
                "id": "evt_123", "slug": "us-fed-rate-cut-2026", "title": "Will the Fed cut rates in 2026?",
                "type": "SINGLE_MARKET", "createdAt": "2026-01-10T09:00:00Z", "imageUrl": "https://cdn.bayse.markets/events/fed.png"
            },
            "market": { "id": "mkt_456", "title": "Yes or No", "imageUrl": null }
        },
        "timestamp": 1700000000000
    }"#;

    let event = WsEvent::from_json(json).expect("should parse buy_order event");
    match event {
        WsEvent::TradeOrder(e) => {
            assert_eq!(e._type, "buy_order");
            assert_eq!(e.data.order.side, "BUY");
            assert_eq!(e.data.order.amount, 100.0);
            assert_eq!(e.data.order.outcome.as_deref(), Some("YES"));
            assert_eq!(e.data.event.slug, "us-fed-rate-cut-2026");
            assert_eq!(e.data.market.id, "mkt_456");
        }
        other => panic!("Expected TradeOrder, got {other:?}"),
    }
}

#[test]
fn test_parse_price_update_event() {
    let json = r#"{
        "type": "price_update",
        "data": {
            "id": "evt_123",
            "slug": "us-fed-rate-cut-2026",
            "title": "Will the Fed cut rates in 2026?",
            "status": "open",
            "type": "SINGLE_MARKET",
            "markets": [
                {
                    "id": "mkt_456",
                    "question": "Yes or No?",
                    "outcomes": ["Yes", "No"],
                    "engine": "CLOB",
                    "prices": { "YES": 0.65, "NO": 0.35 }
                }
            ]
        },
        "timestamp": 1700000000000
    }"#;

    let event = WsEvent::from_json(json).expect("should parse price_update event");
    match event {
        WsEvent::PriceUpdate(e) => {
            assert_eq!(e._type, "price_update");
            assert_eq!(e.data.title, "Will the Fed cut rates in 2026?");
            assert_eq!(e.data.markets.len(), 1);
            assert_eq!(e.data.markets[0].prices.get("YES"), Some(&0.65));
            assert_eq!(e.data.markets[0].prices.get("NO"), Some(&0.35));
        }
        other => panic!("Expected PriceUpdate, got {other:?}"),
    }
}

#[test]
fn test_parse_orderbook_update_event() {
    let json = r#"{
        "type": "orderbook_update",
        "data": {
            "orderbook": {
                "marketId": "mkt_456",
                "outcomeId": "out_789",
                "timestamp": "2026-02-17T10:40:00Z",
                "bids": [
                    { "price": 0.60, "quantity": 500, "total": 300.0 },
                    { "price": 0.55, "quantity": 300, "total": 165.0 }
                ],
                "asks": [
                    { "price": 0.65, "quantity": 200, "total": 130.0 },
                    { "price": 0.70, "quantity": 400, "total": 280.0 }
                ],
                "lastTradedPrice": 0.65,
                "lastTradedSide": "BUY"
            }
        },
        "timestamp": 1700000000000
    }"#;

    let event = WsEvent::from_json(json).expect("should parse orderbook_update event");
    match event {
        WsEvent::OrderbookUpdate(e) => {
            assert_eq!(e._type, "orderbook_update");
            assert_eq!(e.data.orderbook.market_id, "mkt_456");
            assert_eq!(e.data.orderbook.bids.len(), 2);
            assert_eq!(e.data.orderbook.asks.len(), 2);
            assert_eq!(e.data.orderbook.last_traded_price, Some(0.65));
            assert_eq!(e.data.orderbook.bids[0].price, 0.60);
            assert_eq!(e.data.orderbook.bids[0].quantity, 500.0);
        }
        other => panic!("Expected OrderbookUpdate, got {other:?}"),
    }
}

#[test]
fn test_parse_order_updated_event() {
    let json = r#"{
        "type": "order_updated",
        "data": {
            "orderId": "7f5e2a1c-3b4d-4e6f-8a9b-1c2d3e4f5a6b",
            "eventId": "a1b2c3d4-5e6f-7a8b-9c0d-1e2f3a4b5c6d",
            "marketId": "b2c3d4e5-6f7a-8b9c-0d1e-2f3a4b5c6d7e",
            "order": {
                "id": "7f5e2a1c-3b4d-4e6f-8a9b-1c2d3e4f5a6b",
                "userId": "68eea9d8-a0fe-4534-ae88-b71e2f4f5c8f",
                "marketId": "b2c3d4e5-6f7a-8b9c-0d1e-2f3a4b5c6d7e",
                "outcomeId": "c3d4e5f6-7a8b-9c0d-1e2f-3a4b5c6d7e8f",
                "outcomeLabel": "Yes",
                "side": "BUY",
                "price": 0.65,
                "quantity": 150.0,
                "filledQuantity": 100.0,
                "remainingQuantity": 50.0,
                "avgFillPrice": 0.64,
                "status": "PARTIAL_FILLED",
                "timeInForce": "GTC",
                "createdAt": 1700000000,
                "updatedAt": 1700000050
            },
            "timestamp": 1700000050
        },
        "timestamp": 1700000050000
    }"#;

    let event = WsEvent::from_json(json).expect("should parse order_updated event");
    match event {
        WsEvent::OrderUpdated(e) => {
            assert_eq!(e._type, "order_updated");
            assert_eq!(e.data.order_id, "7f5e2a1c-3b4d-4e6f-8a9b-1c2d3e4f5a6b");
            assert_eq!(e.data.order.side, "BUY");
            assert_eq!(e.data.order.filled_quantity, 100.0);
            assert_eq!(e.data.order.remaining_quantity, 50.0);
            assert_eq!(e.data.order.status, "PARTIAL_FILLED");
        }
        other => panic!("Expected OrderUpdated, got {other:?}"),
    }
}

#[test]
fn test_parse_asset_price_event() {
    let json = r#"{
        "type": "asset_price",
        "data": {
            "symbol": "BTCUSDT",
            "price": 67432.15,
            "timestamp": 1700000000000
        },
        "timestamp": 1700000000000
    }"#;

    let event = WsEvent::from_json(json).expect("should parse asset_price event");
    match event {
        WsEvent::AssetPrice(e) => {
            assert_eq!(e._type, "asset_price");
            assert_eq!(e.data.symbol, "BTCUSDT");
            assert_eq!(e.data.price, 67_432.15);
            assert_eq!(e.data.timestamp, 1_700_000_000_000);
        }
        other => panic!("Expected AssetPrice, got {other:?}"),
    }
}

#[test]
fn test_parse_invalid_type_yields_error() {
    let json = r#"{"type": "unknown_event", "data": {}}"#;
    let result = WsEvent::from_json(json);
    assert!(result.is_err(), "expected error for unknown event type");
}

#[test]
fn test_parse_missing_type_yields_error() {
    let json = r#"{"someField": "hello"}"#;
    let result = WsEvent::from_json(json);
    assert!(result.is_err(), "expected error for missing type field");
}

// ---------------------------------------------------------------------------
// WsSubscription serialisation
// ---------------------------------------------------------------------------

#[test]
fn test_subscription_serialise_price_update() {
    let sub = WsSubscription::new("subscribe", "prices").with_event_id("evt_123");
    let json = serde_json::to_string(&sub).expect("should serialise");

    // Should contain the right fields in camelCase
    assert!(json.contains(r#""type":"subscribe""#), "{json}");
    assert!(json.contains(r#""channel":"prices""#), "{json}");
    assert!(json.contains(r#""eventId":"evt_123""#), "{json}");
}

#[test]
fn test_subscription_serialise_orderbook() {
    let sub = WsSubscription::new("subscribe", "orderbook")
        .with_market_ids(vec!["mkt_1".into(), "mkt_2".into()])
        .with_currency("USD");
    let json = serde_json::to_string(&sub).expect("should serialise");

    assert!(json.contains(r#""type":"subscribe""#), "{json}");
    assert!(json.contains(r#""channel":"orderbook""#), "{json}");
    assert!(json.contains(r#""marketIds":["#), "{json}");
    assert!(json.contains(r#""currency":"USD""#), "{json}");
}

#[test]
fn test_subscription_serialise_asset_prices() {
    let sub = WsSubscription::new("subscribe", "asset_prices")
        .with_symbols(vec!["BTCUSDT".into(), "ETHUSDT".into()]);
    let json = serde_json::to_string(&sub).expect("should serialise");

    assert!(json.contains(r#""type":"subscribe""#), "{json}");
    assert!(json.contains(r#""channel":"asset_prices""#), "{json}");
    assert!(json.contains(r#""symbols":["#), "{json}");
}

#[test]
fn test_subscription_serialise_ping() {
    let ping = WsSubscription::ping();
    let json = serde_json::to_string(&ping).expect("should serialise");
    assert_eq!(json, r#"{"type":"ping"}"#);
}

#[test]
fn test_subscription_serialise_unsubscribe() {
    let unsub = WsSubscription::unsubscribe("prices:evt_123");
    let json = serde_json::to_string(&unsub).expect("should serialise");
    assert!(json.contains(r#""type":"unsubscribe""#), "{json}");
    assert!(json.contains(r#""room":"prices:evt_123""#), "{json}");
}

// ---------------------------------------------------------------------------
// WsAuth serialisation
// ---------------------------------------------------------------------------

#[test]
fn test_auth_serialise_api_key() {
    let auth = WsAuth::with_api_key("pk_live_abcdef123456");
    let json = serde_json::to_string(&auth).expect("should serialise");
    assert!(
        json.contains(r#""apiKey":"pk_live_abcdef123456""#),
        "{json}"
    );
    assert!(!json.contains("accessToken"), "{json}");
}

#[test]
fn test_auth_serialise_access_token() {
    let auth = WsAuth::with_access_token("eyJhbGciOiJIUzI1NiIs...", Some("device-123".into()));
    let json = serde_json::to_string(&auth).expect("should serialise");
    assert!(
        json.contains(r#""accessToken":"eyJhbGciOiJIUzI1NiIs...""#),
        "{json}"
    );
    assert!(json.contains(r#""deviceId":"device-123""#), "{json}");
    assert!(!json.contains("apiKey"), "{json}");
}

#[test]
fn test_auth_serialise_access_token_no_device() {
    let auth = WsAuth::with_access_token("eyJhbGciOiJIUzI1NiIs...", None::<String>);
    let json = serde_json::to_string(&auth).expect("should serialise");
    assert!(
        json.contains(r#""accessToken":"eyJhbGciOiJIUzI1NiIs...""#),
        "{json}"
    );
    assert!(!json.contains("deviceId"), "{json}");
}

// ---------------------------------------------------------------------------
// WebSocketHandler trait
// ---------------------------------------------------------------------------

#[test]
fn test_handler_closure_receives_events() {
    let json = r#"{"type":"pong","timestamp":1700000000}"#;

    // A closure that collects received event types
    let mut received_types: Vec<&'static str> = Vec::new();

    {
        let mut handler = |event: WsEvent| -> Result<(), BayseError> {
            match event {
                WsEvent::Pong(_) => received_types.push("pong"),
                WsEvent::Connected(_) => received_types.push("connected"),
                _ => received_types.push("other"),
            }
            Ok(())
        };

        // Use the trait method
        WebSocketHandler::handle_msg(&mut handler, json).expect("handler should process");
    }

    assert_eq!(received_types, vec!["pong"]);
}

#[test]
fn test_handler_closure_breaks_on_error() {
    let json = r#"{"type":"pong","timestamp":1700000000}"#;

    let mut handler = |event: WsEvent| -> Result<(), BayseError> {
        match event {
            WsEvent::Pong(_) => Err(BayseError::Base("stop requested".into())),
            _ => Ok(()),
        }
    };

    let result = WebSocketHandler::handle_msg(&mut handler, json);
    assert!(result.is_err(), "expected error to stop the loop");
}

// ---------------------------------------------------------------------------
// SubscriptionCommand serialisation
// ---------------------------------------------------------------------------

#[test]
fn test_subscription_command_subscribe_serialises() {
    use bayse::ws::SubscriptionCommand;

    let sub = WsSubscription::new("subscribe", "prices").with_event_id("evt_123");
    let cmd = SubscriptionCommand::Subscribe(sub);
    // The command itself is not serialised; it carries the WsSubscription
    match cmd {
        SubscriptionCommand::Subscribe(s) => {
            let json = serde_json::to_string(&s).unwrap();
            assert!(json.contains(r#""channel":"prices""#));
        }
        _ => panic!("expected Subscribe variant"),
    }
}

// ---------------------------------------------------------------------------
// Live connection test (requires BAYSE_API_KEY / BAYSE_WS_HOST env vars)
// ---------------------------------------------------------------------------

/// Connect to the realtime endpoint, subscribe to BTCUSDT, read one
/// message, and disconnect.  This exercises the full WsClient lifecycle.
#[tokio::test]
async fn test_live_connect_and_disconnect() {
    let ws_host = std::env::var("BAYSE_WS_HOST").unwrap_or_else(|_| WS_HOST.to_string());
    let api_key = std::env::var("BAYSE_API_KEY").unwrap_or_default();

    // Skip if no API key is set
    if api_key.is_empty() {
        eprintln!("Skipping live WS test — set BAYSE_API_KEY");
        return;
    }

    let config = Config::default();
    // Override the WS endpoint; keep REST at default
    let client = Client::new(
        Some(api_key),
        None,
        ws_host,
        config.session_token,
        config.device_id,
    );

    // Connect, subscribe, read one frame, disconnect
    let mut ws = bayse::ws::client::WsClient::new(
        client
            .wss_connect("/ws/v1/realtime")
            .await
            .expect("connect"),
    );

    let sub = WsSubscription::new("subscribe", "asset_prices").with_symbols(vec!["BTCUSDT".into()]);
    let sub_msg = serde_json::to_string(&sub).expect("serialise");
    ws.send_text(&sub_msg).await.expect("send subscription");

    // Read at least one line
    let msg = tokio::time::timeout(std::time::Duration::from_secs(100), ws.read_text())
        .await
        .expect("timeout waiting for message")
        .expect("stream ended");

    eprintln!("Received live WS message: {msg:.80}…");
    ws.disconnect().await.expect("disconnect");
}
