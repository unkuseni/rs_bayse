#[cfg(test)]
mod system_tests {
    use super::*;
    use bayse::{Bayse, SystemManager};
    use tokio::test;

    #[test]
    async fn test_health_check() {
        let system: SystemManager = Bayse::new(None, None);

        let res = system.health().await;
        println!("{:?}", res);
    }

    #[test]
    async fn test_version() {
        let system: SystemManager = Bayse::new(None, None);

        let res = system.version().await;
        println!("{:?}", res);
    }
}
