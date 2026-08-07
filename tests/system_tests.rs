#[cfg(test)]
mod system_tests {
    use bayse::{Bayse, SystemManager};
    use tokio::test;

    // NOTE: Live smoke tests against the production API. Ignored by default
    // so `cargo test` stays hermetic; run with `cargo test -- --ignored`.
    #[test]
    #[ignore]
    async fn test_health_check() {
        let system: SystemManager = Bayse::new(None, None);

        let res = system.health().await;
        println!("{:?}", res);
    }

    #[test]
    #[ignore]
    async fn test_version() {
        let system: SystemManager = Bayse::new(None, None);

        let res = system.version().await;
        println!("{:?}", res);
    }
}
