use bayse::{Bayse, WalletManager};

const API_KEY: &str = "";
const SECRET_KEY: &str = "";

#[tokio::test]
async fn test_get_assets() {
    let wallet: WalletManager = Bayse::new(Some(API_KEY.to_string()), Some(SECRET_KEY.to_string()));

    match wallet.get_assets().await {
        Ok(assets) => {
            println!("Assets: {:#?}", assets);
        }
        Err(err) => {
            assert!(false, "Failed to get assets: {:?}", err);
        }
    }
}
