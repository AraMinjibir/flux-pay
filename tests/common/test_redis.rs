use redis::Client;

#[allow(dead_code)]
pub struct TestRedis {
    pub client: Client,
}

#[allow(dead_code)]
impl TestRedis {
    pub async fn new() -> Self {
        dotenvy::dotenv().ok();

        let redis_url = std::env::var("TEST_REDIS_URL")
            .unwrap_or_else(|_| "redis://127.0.0.1:6379".to_string());

        let client = Client::open(redis_url).expect("Unable to create Redis client");

        Self { client }
    }

    pub async fn reset(&self) {
        let mut conn = self
            .client
            .get_multiplexed_async_connection()
            .await
            .expect("Unable to connect to Redis");

        redis::cmd("FLUSHDB")
            .query_async::<()>(&mut conn)
            .await
            .expect("Unable to reset Redis");
    }
}
