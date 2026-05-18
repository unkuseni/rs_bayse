use bayse::{
    BatchAmendOrdersItem, BatchAmendOrdersRequest, BatchCancelOrdersRequest, BatchOrderItem,
    BatchPlaceOrdersRequest, Bayse, GetPnLQuery, GetQuoteRequest, ListOrdersQuery, MintBurnRequest,
    PlaceOrderRequest, TradingManager,
};

const API_KEY: &str = "";
const SECRET_KEY: &str = "";

// ---------------------------------------------------------------------------
// Events
// ---------------------------------------------------------------------------

#[tokio::test]
async fn test_list_events() {
    let trader: TradingManager = Bayse::new(Some(API_KEY.into()), Some(SECRET_KEY.into()));
    match trader.list_events(None, Some(3)).await {
        Ok(resp) => {
            println!(
                "Got {} events (page {}/{}, total {})",
                resp.events.len(),
                resp.pagination.page,
                resp.pagination.last_page,
                resp.pagination.total_count,
            );
            for event in &resp.events {
                println!(
                    "  · {} [{}] ({} markets, status: {})",
                    event.title,
                    event.category,
                    event.markets.len(),
                    event.status,
                );
            }
        }
        Err(e) => panic!("{:?}", e),
    }
}

#[tokio::test]
async fn test_filter_for_crypto() {
    let trader: TradingManager = Bayse::new(Some(API_KEY.into()), Some(SECRET_KEY.into()));
    match trader.list_events(None, None).await {
        Ok(resp) => resp
            .events
            .iter()
            .filter(|e| e.category == "CRYPTO")
            .for_each(|e| {
                println!("  · {:#?} [{:#?}] Slug: {:#?}", e.title, e.category, e.slug);
            }),
        Err(e) => panic!("{:?}", e),
    }
}

#[tokio::test]
async fn test_get_event() {
    let trader: TradingManager = Bayse::new(Some(API_KEY.into()), Some(SECRET_KEY.into()));
    let event_id = "";
    match trader.get_event(event_id).await {
        Ok(event) => println!("{:#?}", event),
        Err(e) => panic!("{:?}", e),
    }
}

#[tokio::test]
async fn test_get_event_by_slug() {
    let trader: TradingManager = Bayse::new(Some(API_KEY.into()), Some(SECRET_KEY.into()));
    let slug = "";
    match trader.get_event_by_slug(slug).await {
        Ok(event) => println!("{:#?}", event),
        Err(e) => panic!("{:?}", e),
    }
}

#[tokio::test]
async fn test_list_series() {
    let trader: TradingManager = Bayse::new(Some(API_KEY.into()), Some(SECRET_KEY.into()));
    match trader.list_series(None, None).await {
        Ok(series) => println!("{:#?}", series),
        Err(e) => panic!("{:?}", e),
    }
}

#[tokio::test]
async fn test_get_series_events() {
    let trader: TradingManager = Bayse::new(Some(API_KEY.into()), Some(SECRET_KEY.into()));
    let series_id = "";
    match trader.get_series_events(series_id, None, None).await {
        Ok(events) => println!("{:#?}", events),
        Err(e) => panic!("{:?}", e),
    }
}

// ---------------------------------------------------------------------------
// Quotes & Orders
// ---------------------------------------------------------------------------

#[tokio::test]
async fn test_get_quote() {
    let trader: TradingManager = Bayse::new(Some(API_KEY.into()), Some(SECRET_KEY.into()));
    let (event_id, market_id) = ("", "");
    let req = GetQuoteRequest {
        side: "BUY".into(),
        outcome_id: "".into(),
        amount: 100.0,
        currency: Some("USD".into()),
    };
    match trader.get_quote(event_id, market_id, &req).await {
        Ok(quote) => println!("{:#?}", quote),
        Err(e) => panic!("{:?}", e),
    }
}

#[tokio::test]
async fn test_place_order() {
    let trader: TradingManager = Bayse::new(Some(API_KEY.into()), Some(SECRET_KEY.into()));
    let (event_id, market_id) = ("", "");
    let req = PlaceOrderRequest {
        side: "BUY".into(),
        outcome_id: "".into(),
        amount: 10.0,
        order_type: "MARKET".into(),
        currency: Some("USD".into()),
        price: None,
        time_in_force: None,
        post_only: None,
        stp_mode: None,
        max_slippage: None,
        expires_at: None,
    };
    match trader.place_order(event_id, market_id, &req).await {
        Ok(order) => println!("{:#?}", order),
        Err(e) => panic!("{:?}", e),
    }
}

#[tokio::test]
async fn test_batch_place_orders() {
    let trader: TradingManager = Bayse::new(Some(API_KEY.into()), Some(SECRET_KEY.into()));
    let req = BatchPlaceOrdersRequest {
        orders: vec![BatchOrderItem {
            outcome_id: "".into(),
            side: "BUY".into(),
            order_type: "MARKET".into(),
            amount: 10.0,
            currency: Some("USD".into()),
            price: None,
            time_in_force: None,
            post_only: None,
            max_slippage: None,
            expires_at: None,
            stp_mode: None,
            client_order_id: None,
        }],
    };
    match trader.batch_place_orders(&req).await {
        Ok(resp) => println!("{:#?}", resp),
        Err(e) => panic!("{:?}", e),
    }
}

// ---------------------------------------------------------------------------
// Portfolio & PnL
// ---------------------------------------------------------------------------

#[tokio::test]
async fn test_get_portfolio() {
    let trader: TradingManager = Bayse::new(Some(API_KEY.into()), Some(SECRET_KEY.into()));
    match trader.get_portfolio().await {
        Ok(portfolio) => {
            println!(
                "Portfolio: cost={}, current_value={}, change={}%",
                portfolio.portfolio_cost,
                portfolio.portfolio_current_value,
                portfolio.portfolio_percentage_change,
            );
            for position in &portfolio.outcome_balances {
                println!(
                    "  · {} {} (balance={}, value={})",
                    position.market.title,
                    position.outcome,
                    position.balance,
                    position.current_value,
                );
            }
        }
        Err(e) => panic!("{:?}", e),
    }
}

#[tokio::test]
async fn test_get_pnl() {
    let trader: TradingManager = Bayse::new(Some(API_KEY.into()), Some(SECRET_KEY.into()));
    let query = GetPnLQuery {
        time_period: Some("1Y".into()),
        breakdown: Some(true),
        currency: Some("NGN".into()),
        ..Default::default()
    };
    match trader.get_pnl(&query).await {
        Ok(pnl) => {
            println!(
                "PnL: realized={}, wins={}, losses={} trade_pnl={}",
                pnl.realized_pnl, pnl.wins, pnl.losses, pnl.trade_pnl
            );
            if let Some(breakdown) = &pnl.breakdown {
                for item in breakdown {
                    println!("  · {}: {}", item.event_title, item.realized_pnl);
                }
            }
        }
        Err(e) => panic!("{:?}", e),
    }
}

// ---------------------------------------------------------------------------
// Orders
// ---------------------------------------------------------------------------

#[tokio::test]
async fn test_list_orders() {
    let trader: TradingManager = Bayse::new(Some(API_KEY.into()), Some(SECRET_KEY.into()));
    let query = ListOrdersQuery {
        page: Some(1),
        size: Some(10),
        ..Default::default()
    };
    match trader.list_orders(&query).await {
        Ok(resp) => {
            println!(
                "Got {} orders (page {}/{}, total {})",
                resp.orders.len(),
                resp.pagination.page,
                resp.pagination.last_page,
                resp.pagination.total_count,
            );
            for order in &resp.orders {
                println!(
                    "  · {} {} {} (status={})",
                    order.id,
                    order.side,
                    order.outcome.as_deref().unwrap_or("?"),
                    order.status,
                );
            }
        }
        Err(e) => panic!("{:?}", e),
    }
}

#[tokio::test]
async fn test_get_order() {
    let trader: TradingManager = Bayse::new(Some(API_KEY.into()), Some(SECRET_KEY.into()));
    let order_id = "";
    match trader.get_order(order_id).await {
        Ok(order) => println!("{:#?}", order),
        Err(e) => panic!("{:?}", e),
    }
}

#[tokio::test]
async fn test_cancel_order() {
    let trader: TradingManager = Bayse::new(Some(API_KEY.into()), Some(SECRET_KEY.into()));
    let order_id = "";
    match trader.cancel_order(order_id).await {
        Ok(resp) => println!("Cancel response: {}", resp.message),
        Err(e) => panic!("{:?}", e),
    }
}

#[tokio::test]
async fn test_batch_cancel_orders() {
    let trader: TradingManager = Bayse::new(Some(API_KEY.into()), Some(SECRET_KEY.into()));
    let req = BatchCancelOrdersRequest { order_ids: vec![] };
    match trader.batch_cancel_orders(&req).await {
        Ok(resp) => {
            println!(
                "Batch cancel: {}/{} succeeded",
                resp.summary.succeeded, resp.summary.total,
            );
        }
        Err(e) => panic!("{:?}", e),
    }
}

/// ------------------------------------------------------------------
/// Batch Amend Orders
/// ------------------------------------------------------------------

#[tokio::test]
async fn test_batch_amend_orders() {
    let trader: TradingManager = Bayse::new(Some(API_KEY.into()), Some(SECRET_KEY.into()));
    let req = BatchAmendOrdersRequest {
        items: vec![BatchAmendOrdersItem {
            order_id: "".into(),
            new_price: Some(0.50),
            new_size: Some(15.0),
        }],
    };
    match trader.batch_amend_orders(&req).await {
        Ok(resp) => {
            println!(
                "Batch amend: {}/{} succeeded",
                resp.summary.succeeded, resp.summary.total,
            );
        }
        Err(e) => panic!("{:?}", e),
    }
}

// ------------------------------------------------------------------
// Mint / Burn / Activities
// ------------------------------------------------------------------

#[tokio::test]
async fn test_mint_shares() {
    let trader: TradingManager = Bayse::new(Some(API_KEY.into()), Some(SECRET_KEY.into()));
    let market_id = "";
    let req = MintBurnRequest {
        quantity: 10.0,
        currency: Some("USD".into()),
    };
    match trader.mint_shares(market_id, &req).await {
        Ok(resp) => println!("{:#?}", resp),
        Err(e) => panic!("{:?}", e),
    }
}

#[tokio::test]
async fn test_burn_shares() {
    let trader: TradingManager = Bayse::new(Some(API_KEY.into()), Some(SECRET_KEY.into()));
    let market_id = "";
    let req = MintBurnRequest {
        quantity: 10.0,
        currency: Some("USD".into()),
    };
    match trader.burn_shares(market_id, &req).await {
        Ok(resp) => println!("{:#?}", resp),
        Err(e) => panic!("{:?}", e),
    }
}

#[tokio::test]
async fn test_get_activities() {
    let trader: TradingManager = Bayse::new(Some(API_KEY.into()), Some(SECRET_KEY.into()));
    match trader.get_activities(Some(1), Some(10)).await {
        Ok(activities) => println!("{:#?}", activities),
        Err(e) => panic!("{:?}", e),
    }
}
