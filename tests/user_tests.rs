use bayse::{Bayse, UserManager};

// NOTE: These are live integration tests against the production Bayse API.
// They are ignored by default so `cargo test` stays hermetic; run them with
// real credentials via:
//     cargo test --test user_tests -- --ignored
const API_KEY: &str = "";
const SECRET_KEY: &str = "";

const TEST_EMAIL: &str = "johndoe@example.com";
const TEST_PASSWORD: &str = "password123";

#[tokio::test]
#[ignore]
async fn test_lookup_user() {
    // lookup_user requires read authentication (X-Public-Key header) —
    // no login is needed, just a valid API key.
    let manager = UserManager::new(Some(API_KEY.to_string()), Some(SECRET_KEY.to_string()));

    match manager.lookup_user(Some("janedoe"), None).await {
        Ok(user) => {
            println!("User: {:#?}", user);
        }
        Err(err) => panic!("lookup_user failed: {:?}", err),
    }
}

#[tokio::test]
#[ignore]
async fn test_login() {
    let mut manager = UserManager::new(Some(API_KEY.to_string()), Some(SECRET_KEY.to_string()));
    match manager.login(TEST_EMAIL, TEST_PASSWORD).await {
        Ok(user) => {
            println!("User: {:#?}", user);
        }
        Err(err) => panic!("login failed: {:?}", err),
    }
}

#[tokio::test]
#[ignore]
async fn test_create_api_key() {
    let mut manager = UserManager::new(Some(API_KEY.to_string()), Some(SECRET_KEY.to_string()));
    match manager
        .login_and_create_api_key(TEST_EMAIL, TEST_PASSWORD, "david_testing_key")
        .await
    {
        Ok(key) => {
            println!("API Key: {:#?}", key);
        }
        Err(err) => panic!("create_api_key failed: {:?}", err),
    }
}

#[tokio::test]
#[ignore]
async fn test_list_api_keys() {
    let mut manager = UserManager::new(Some(API_KEY.to_string()), Some(SECRET_KEY.to_string()));

    // Log in first.  login() already persists the session token on the client.
    manager
        .login(TEST_EMAIL, TEST_PASSWORD)
        .await
        .expect("login should succeed");

    match manager.list_api_keys().await {
        Ok(keys) => {
            println!("API Keys: {:#?}", keys);
        }
        Err(err) => panic!("list_api_keys failed: {:?}", err),
    }
}

#[tokio::test]
#[ignore]
async fn test_revoke_api_key() {
    // Create a throwaway key, capture its ID, then revoke it.
    // login_and_create_api_key handles both login + key creation,
    // and the session is persisted automatically.
    let mut manager = UserManager::new(Some(API_KEY.to_string()), Some(SECRET_KEY.to_string()));
    let key = manager
        .login_and_create_api_key(TEST_EMAIL, TEST_PASSWORD, "revoke_test_key")
        .await
        .expect("create key should succeed");

    let key_id = key.id.as_str();

    match manager.revoke_api_key(key_id).await {
        Ok(result) => {
            println!("Revoked key {key_id}: {:#?}", result);
        }
        Err(err) => panic!("revoke_api_key failed: {:?}", err),
    }
}

#[tokio::test]
#[ignore]
async fn test_rotate_api_key() {
    // Create a throwaway key, capture its ID, then rotate its secret.
    let mut manager = UserManager::new(Some(API_KEY.to_string()), Some(SECRET_KEY.to_string()));
    let key = manager
        .login_and_create_api_key(TEST_EMAIL, TEST_PASSWORD, "rotate_test_key")
        .await
        .expect("create key should succeed");

    let key_id = key.id.as_str();

    match manager.rotate_api_key(key_id).await {
        Ok(result) => {
            println!("Rotated key {key_id}: {:#?}", result);
        }
        Err(err) => panic!("rotate_api_key failed: {:?}", err),
    }
}
