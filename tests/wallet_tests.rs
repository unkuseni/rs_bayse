use bayse::{Bayse, WalletManager};

// NOTE: Live integration test against the production Bayse API. Ignored by
// default so `cargo test` stays hermetic; run with a real API key via:
//     cargo test --test wallet_tests -- --ignored
const API_KEY: &str = "";
const SECRET_KEY: &str = "";

#[tokio::test]
#[ignore]
async fn test_get_assets() {
    let wallet: WalletManager = Bayse::new(Some(API_KEY.to_string()), Some(SECRET_KEY.to_string()));

    match wallet.get_assets().await {
        Ok(assets) => {
            println!("Assets: {:#?}", assets);
        }
        Err(err) => {
            panic!("Failed to get assets: {:?}", err);
        }
    }
}
