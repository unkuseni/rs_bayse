use bayse::{Bayse, MarketMakerManager};

const API_KEY: &str = "";
const SECRET_KEY: &str = "";

#[tokio::test]
async fn test_get_liquidity_rewards() {
    let mm: MarketMakerManager = Bayse::new(Some(API_KEY.into()), Some(SECRET_KEY.into()));
    match mm.get_liquidity_rewards(Some(1), Some(10)).await {
        Ok(resp) => {
            println!(
                "Got {} liquidity rewards (page {}/{}, total {})",
                resp.data.len(),
                resp.pagination.page,
                resp.pagination.last_page,
                resp.pagination.total_count,
            );
            for record in &resp.data {
                println!(
                    "  · epoch {} — payout={}, status={}",
                    record.epoch_id, record.payout, record.status,
                );
            }
        }
        Err(e) => panic!("{:?}", e),
    }
}

#[tokio::test]
async fn test_get_active_liquidity_rewards() {
    let mm: MarketMakerManager = Bayse::new(Some(API_KEY.into()), Some(SECRET_KEY.into()));
    match mm.get_active_liquidity_rewards().await {
        Ok(resp) => {
            println!("Got {} active liquidity rewards", resp.data.len());
            for reward in &resp.data {
                println!(
                    "  · epoch {} — estimated_payout={}",
                    reward.epoch_id, reward.estimated_payout,
                );
            }
        }
        Err(e) => panic!("{:?}", e),
    }
}

#[tokio::test]
async fn test_get_maker_rebates() {
    let mm: MarketMakerManager = Bayse::new(Some(API_KEY.into()), Some(SECRET_KEY.into()));
    match mm.get_maker_rebates(Some(1), Some(10)).await {
        Ok(resp) => {
            println!(
                "Got {} maker rebates (page {}/{}, total {})",
                resp.data.len(),
                resp.pagination.page,
                resp.pagination.last_page,
                resp.pagination.total_count,
            );
            for record in &resp.data {
                println!(
                    "  · epoch {} — rebate={}, status={}",
                    record.epoch_id, record.rebate_amount, record.status,
                );
            }
        }
        Err(e) => panic!("{:?}", e),
    }
}

#[tokio::test]
async fn test_get_active_maker_rebates() {
    let mm: MarketMakerManager = Bayse::new(Some(API_KEY.into()), Some(SECRET_KEY.into()));
    match mm.get_active_maker_rebates().await {
        Ok(resp) => {
            println!("Got {} active maker rebates", resp.data.len());
            for rebate in &resp.data {
                println!(
                    "  · epoch {} — maker_volume={}, rebate={}",
                    rebate.epoch_id, rebate.maker_volume, rebate.rebate_amount,
                );
            }
        }
        Err(e) => panic!("{:?}", e),
    }
}
