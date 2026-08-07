//! Hermetic tests for the friendly typed response types.
//!
//! Each test feeds the *exact* example JSON from the official docs
//! (https://docs.bayse.markets) into the typed structs to prove the
//! "clean up the data" helpers parse real response shapes correctly.
//! No network access required.

use bayse::{
    ActivitiesResponse, EventSeries, ListSeriesResponse, OrderBook, PaginatedResponse, PricePoint,
    SeriesEventSummary, SportsGamesResponse, SportsLeaguesResponse, SportsTeamsResponse, Ticker,
    Trade, UserProfile,
};

fn parse<T: serde::de::DeserializeOwned>(json: &str) -> T {
    serde_json::from_str(json).expect("docs example should parse into typed struct")
}

#[test]
fn ticker_parses_docs_example() {
    let json = r#"
    {
      "marketId": "b2c3d4e5-f6a7-8901-bcde-f12345678901",
      "outcome": "YES",
      "lastPrice": 0.72,
      "bestBid": 0.70,
      "bestAsk": 0.72,
      "midPrice": 0.71,
      "spread": 0.02,
      "volume24h": 15420,
      "high24h": 0.74,
      "low24h": 0.65,
      "priceChange24h": 0.04,
      "tradeCount24h": 247,
      "timestamp": "2026-02-17T12:00:00Z"
    }
    "#;
    let ticker: Ticker = parse(json);
    assert_eq!(ticker.market_id, "b2c3d4e5-f6a7-8901-bcde-f12345678901");
    assert_eq!(ticker.outcome, "YES");
    assert_eq!(ticker.last_price, 0.72);
    assert_eq!(ticker.best_bid, 0.70);
    assert_eq!(ticker.best_ask, 0.72);
    assert_eq!(ticker.mid_price, 0.71);
    assert_eq!(ticker.spread, 0.02);
    assert_eq!(ticker.volume_24h, 15420.0);
    assert_eq!(ticker.high_24h, 0.74);
    assert_eq!(ticker.low_24h, 0.65);
    assert_eq!(ticker.price_change_24h, 0.04);
    assert_eq!(ticker.trade_count_24h, 247);
    assert_eq!(ticker.timestamp, "2026-02-17T12:00:00Z");
}

#[test]
fn trades_parse_docs_example() {
    let json = r#"
    {
      "data": [
        {
          "id": "t1a2b3c4-d5e6-7890-abcd-ef1234567890",
          "marketId": "b2c3d4e5-f6a7-8901-bcde-f12345678901",
          "outcome": "YES",
          "price": 0.72,
          "size": 100,
          "createdAt": "2026-02-17T12:00:01Z"
        },
        {
          "id": "t2b3c4d5-e6f7-8901-bcde-f12345678901",
          "marketId": "b2c3d4e5-f6a7-8901-bcde-f12345678901",
          "outcome": "NO",
          "price": 0.28,
          "size": 250,
          "createdAt": "2026-02-17T11:59:45Z"
        }
      ],
      "pagination": {
        "page": 1,
        "size": 20,
        "totalCount": 2,
        "lastPage": 1
      }
    }
    "#;
    let resp: PaginatedResponse<Trade> = parse(json);
    assert_eq!(resp.data.len(), 2);
    assert_eq!(resp.pagination.page, 1);
    assert_eq!(resp.pagination.last_page, 1);
    assert_eq!(resp.pagination.total_count, 2);
    let first = &resp.data[0];
    assert_eq!(first.id, "t1a2b3c4-d5e6-7890-abcd-ef1234567890");
    assert_eq!(first.outcome, "YES");
    assert_eq!(first.price, 0.72);
    assert_eq!(first.size, 100.0);
    assert_eq!(first.created_at, "2026-02-17T12:00:01Z");
}

#[test]
fn order_books_parse_docs_example() {
    let json = r#"
    [
      {
        "marketId": "660e8400-e29b-41d4-a716-446655440001",
        "outcomeId": "c3d4e5f6-a7b8-9012-cdef-123456789012",
        "timestamp": "2025-01-15T10:30:00Z",
        "bids": [
          { "price": 0.70, "quantity": 200, "total": 140.0 },
          { "price": 0.69, "quantity": 450, "total": 310.5 }
        ],
        "asks": [
          { "price": 0.72, "quantity": 150, "total": 108.0 },
          { "price": 0.73, "quantity": 300, "total": 219.0 }
        ],
        "lastTradedPrice": 0.71,
        "lastTradedSide": "BUY"
      }
    ]
    "#;
    let books: Vec<OrderBook> = parse(json);
    assert_eq!(books.len(), 1);
    let book = &books[0];
    assert_eq!(book.market_id, "660e8400-e29b-41d4-a716-446655440001");
    assert_eq!(book.outcome_id, "c3d4e5f6-a7b8-9012-cdef-123456789012");
    assert_eq!(book.bids.len(), 2);
    assert_eq!(book.bids[0].price, 0.70);
    assert_eq!(book.bids[0].quantity, 200.0);
    assert_eq!(book.bids[0].total, 140.0);
    assert_eq!(book.asks[1].price, 0.73);
    assert_eq!(book.last_traded_price, Some(0.71));
    assert_eq!(book.last_traded_side.as_deref(), Some("BUY"));
}

#[test]
fn price_history_parses_docs_example() {
    let json = r#"
    {
      "b2c3d4e5-f6a7-8901-bcde-f12345678901": [
        { "outcome": "YES", "price": 0.65, "timestamp": "2026-02-10T00:00:00Z" },
        { "outcome": "YES", "price": 0.68, "timestamp": "2026-02-11T00:00:00Z" },
        { "outcome": "YES", "price": 0.72, "timestamp": "2026-02-17T00:00:00Z" }
      ]
    }
    "#;
    let history: std::collections::BTreeMap<String, Vec<PricePoint>> = parse(json);
    assert_eq!(history.len(), 1);
    let points = &history["b2c3d4e5-f6a7-8901-bcde-f12345678901"];
    assert_eq!(points.len(), 3);
    assert_eq!(points[0].outcome, "YES");
    assert_eq!(points[0].price, 0.65);
    assert_eq!(points[2].price, 0.72);
}

#[test]
fn user_profile_parses_docs_example() {
    let json = r#"
    {
      "id": "68eea9d8-a0fe-4534-ae88-b71e2f4f5c8f",
      "tag": "mulumba",
      "imageUrl": "https://cdn.bayse.markets/profile-images/mulumba.jpg"
    }
    "#;
    let profile: UserProfile = parse(json);
    assert_eq!(profile.id, "68eea9d8-a0fe-4534-ae88-b71e2f4f5c8f");
    assert_eq!(profile.tag, "mulumba");
    assert_eq!(
        profile.image_url,
        "https://cdn.bayse.markets/profile-images/mulumba.jpg"
    );
}

#[test]
fn series_parse_docs_example() {
    let json = r#"
    {
      "series": [
        {
          "id": "f1e2d3c4-b5a6-7890-abcd-ef1234567890",
          "slug": "crypto-btc-1h",
          "displayName": "Bitcoin Hourly Markets",
          "description": "Bitcoin price prediction markets that run every hour.",
          "category": "CRYPTO",
          "intervalType": "HOURLY",
          "assetSymbol": "BTC",
          "automationType": "CRYPTO_PRICE_UP_DOWN_HOURLY"
        }
      ],
      "pagination": {
        "page": 1,
        "size": 20,
        "lastPage": 1,
        "totalCount": 1
      }
    }
    "#;
    let resp: ListSeriesResponse = parse(json);
    assert_eq!(resp.series.len(), 1);
    let series: &EventSeries = &resp.series[0];
    assert_eq!(series.slug, "crypto-btc-1h");
    assert_eq!(series.display_name, "Bitcoin Hourly Markets");
    assert_eq!(series.interval_type, "HOURLY");
    assert_eq!(series.asset_symbol, "BTC");
}

#[test]
fn series_events_parse_docs_example() {
    let json = r#"
    [
      {
        "id": "a1b2c3d4-e5f6-7890-abcd-ef1234567890",
        "title": "Bitcoin Hourly — Feb 24 11am GMT",
        "openingDate": "2025-02-24T11:00:00Z",
        "closingDate": "2025-02-24T12:00:00Z",
        "resolutionDate": "2025-02-24T12:01:00Z"
      },
      {
        "id": "b2c3d4e5-f6a7-8901-bcde-f12345678901",
        "title": "Bitcoin Hourly — Feb 24 12pm GMT",
        "openingDate": "2025-02-24T12:00:00Z",
        "closingDate": "2025-02-24T13:00:00Z",
        "resolutionDate": "2025-02-24T13:01:00Z"
      }
    ]
    "#;
    let events: Vec<SeriesEventSummary> = parse(json);
    assert_eq!(events.len(), 2);
    assert_eq!(events[0].title, "Bitcoin Hourly — Feb 24 11am GMT");
    assert_eq!(events[0].closing_date, "2025-02-24T12:00:00Z");
}

#[test]
fn sports_parse_docs_example() {
    // Games (fields per list-sports-games docs)
    let games_json = r#"
    {
      "games": [
        {
          "id": "g1",
          "slug": "bm-game-20260413-rma-ath",
          "sport": "soccer",
          "homeTeamId": "t1",
          "awayTeamId": "t2",
          "startDate": "2026-04-13T19:00:00Z",
          "league": "Spain - La Liga",
          "isLive": false,
          "isPopular": true,
          "homeTeam": {
            "id": "t1",
            "sport": "soccer",
            "name": "Real Madrid",
            "slug": "real-madrid",
            "shortCode": "RMA",
            "league": "Spain - La Liga",
            "imageUrl": "https://cdn.bayse.markets/teams/rma.png",
            "isPopular": true
          },
          "awayTeam": {
            "id": "t2",
            "sport": "soccer",
            "name": "Athletic Club",
            "slug": "athletic-club",
            "shortCode": "ATH",
            "league": "Spain - La Liga",
            "imageUrl": null,
            "isPopular": false
          }
        }
      ],
      "pagination": { "page": 1, "size": 50, "lastPage": 1, "totalCount": 1 }
    }
    "#;
    let games: SportsGamesResponse = parse(games_json);
    assert_eq!(games.games.len(), 1);
    let game = &games.games[0];
    assert_eq!(game.slug, "bm-game-20260413-rma-ath");
    assert_eq!(game.league, "Spain - La Liga");
    assert_eq!(game.home_team.as_ref().unwrap().name, "Real Madrid");
    assert_eq!(game.away_team.as_ref().unwrap().short_code, "ATH");
    assert!(game.is_popular);

    // Teams (fields per list-sports-teams docs)
    let teams_json = r#"
    {
      "teams": [
        {
          "id": "t1",
          "sport": "soccer",
          "name": "Real Madrid",
          "slug": "real-madrid",
          "shortCode": "RMA",
          "league": "Spain - La Liga",
          "imageUrl": "https://cdn.bayse.markets/teams/rma.png",
          "isPopular": true
        }
      ],
      "pagination": { "page": 1, "size": 50, "lastPage": 1, "totalCount": 1 }
    }
    "#;
    let teams: SportsTeamsResponse = parse(teams_json);
    assert_eq!(teams.teams[0].name, "Real Madrid");
    assert_eq!(teams.teams[0].short_code, "RMA");

    // Leagues (fields per list-sports-leagues docs)
    let leagues_json = r#"
    {
      "leagues": [
        {
          "name": "England - Premier League",
          "shortName": "EPL",
          "imageUrl": "https://cdn.bayse.markets/leagues/epl.png"
        }
      ]
    }
    "#;
    let leagues: SportsLeaguesResponse = parse(leagues_json);
    assert_eq!(leagues.leagues[0].name, "England - Premier League");
    assert_eq!(leagues.leagues[0].short_name, "EPL");
}

#[test]
fn activities_parse_docs_example() {
    let json = r#"
    {
      "activities": [
        {
          "id": "a1b2c3d4-e5f6-7890-abcd-ef1234567890",
          "type": "BUY_MARKET_ORDER_CREATED",
          "eventId": "c3d4e5f6-a7b8-9012-cdef-123456789012",
          "marketId": "b2c3d4e5-f6a7-8901-bcde-f12345678901",
          "outcomeId": "d4e5f6a7-b8c9-0123-def1-234567890123",
          "orderId": "e5f6a7b8-c9d0-1234-ef12-345678901234",
          "eventType": "SINGLE",
          "imageUrl": "https://example.com/image.png",
          "eventTitle": "Will BTC reach $100k by March 2026?",
          "marketTitle": "Bitcoin Price Prediction",
          "outcome": "YES",
          "currency": "USD",
          "amount": "100",
          "fee": "2",
          "size": "138.21",
          "price": "0.7235",
          "totalCost": "102",
          "currencyBaseMultiplier": "1",
          "status": "FILLED",
          "createdAt": "2026-02-17T12:00:00Z",
          "updatedAt": "2026-02-17T12:00:00Z"
        }
      ],
      "pagination": {
        "page": 1,
        "size": 20,
        "lastPage": 4,
        "totalCount": 73
      }
    }
    "#;
    let resp: ActivitiesResponse = parse(json);
    assert_eq!(resp.activities.len(), 1);
    let activity = &resp.activities[0];
    assert_eq!(activity.activity_type, "BUY_MARKET_ORDER_CREATED");
    assert_eq!(activity.event_title.as_deref(), Some("Will BTC reach $100k by March 2026?"));
    assert_eq!(activity.outcome.as_deref(), Some("YES"));
    assert_eq!(activity.amount.as_deref(), Some("100"));
    assert_eq!(activity.fee.as_deref(), Some("2"));
    assert_eq!(activity.size.as_deref(), Some("138.21"));
    assert_eq!(activity.price.as_deref(), Some("0.7235"));
    assert_eq!(resp.pagination.total_count, 73);
    assert_eq!(resp.pagination.last_page, 4);
}
