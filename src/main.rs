use async_trait::async_trait;
use futures::stream::BoxStream;
use std::{
    collections::HashMap,
    result::Result,
    sync::{Arc, Mutex},
};

type City = String;
type Temperature = u64;

#[async_trait]
trait Api: Send + Sync {
    async fn fetch(&self) -> Result<HashMap<City, Temperature>, String>;
    async fn subscribe(&self) -> BoxStream<Result<(City, Temperature), String>>;
}

struct StreamCache {
    results: Arc<Mutex<HashMap<String, u64>>>,
}

impl StreamCache {
    fn new(api: Box<dyn Api>) -> Self {
        let instance = Self {
            results: Arc::new(Mutex::new(HashMap::new())),
        };
        instance.update_in_background(api);
        instance
    }

    fn get(&self, key: &str) -> Option<u64> {
        let results = self.results.lock().expect("poisoned");
        results.get(key).copied()
    }

    fn update_in_background(&self, api: Box<dyn Api>) {
        tokio::task::spawn(async move {
            // TODO implement
        });
    }
}

mod tests {
    use std::time::Duration;
    use tokio::time;

    use futures::{stream::select, StreamExt};
    use maplit::hashmap;

    use super::*;

    struct TestApi {}
    #[async_trait]
    impl Api for TestApi {
        async fn fetch(&self) -> Result<HashMap<City, Temperature>, String> {
            time::sleep(Duration::from_millis(10)).await;
            Ok(hashmap! {
                "Berlin".to_string() => 29,
                "Paris".to_string() => 31,
            })
        }
        async fn subscribe(&self) -> BoxStream<Result<(City, Temperature), String>> {
            let results = vec![
                Ok(("London".to_string(), 27)),
                Ok(("Paris".to_string(), 32)),
            ];
            select(futures::stream::iter(results), futures::stream::pending()).boxed()
        }
    }
    #[tokio::test]
    async fn works() {
        let cache = StreamCache::new(Box::new(TestApi {}));

        // Allow cache to update
        time::sleep(Duration::from_millis(100)).await;

        assert_eq!(cache.get("Berlin"), Some(29));
        assert_eq!(cache.get("London"), Some(27));
        assert_eq!(cache.get("Paris"), Some(32));
    }
}
