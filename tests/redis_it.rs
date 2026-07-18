mod common;
use crate::common::test_redis::TestRedis;
use flux_pay::{
    domain::{
        idempotency::{
            models::{ReservationResult, StoredResponse},
            repository::IdempotencyRepository,
        },
        payment::status::PaymentStatus,
    },
    infrastructure::redis::redis_idempotency_repository::RedisIdempotencyRepository,
};
use uuid::Uuid;

#[tokio::test]
async fn should_reserve_new_key() {
    // Arrange

    let redis = TestRedis::new().await;

    redis.reset().await;

    let repository = RedisIdempotencyRepository::new(redis.client);

    let key = Uuid::new_v4();

    // Act

    let result = repository.reserve(key).await.unwrap();

    // Assert

    assert!(matches!(result, ReservationResult::Reserved));
}

#[tokio::test]
async fn should_return_in_progress_for_duplicate_key() {
    let redis = TestRedis::new().await;

    redis.reset().await;

    let repository = RedisIdempotencyRepository::new(redis.client);

    let key = Uuid::new_v4();

    repository.reserve(key).await.unwrap();

    let result = repository.reserve(key).await.unwrap();

    assert!(matches!(result, ReservationResult::InProgress));
}

#[tokio::test]
async fn should_return_completed_response_after_completion() {
    let redis = TestRedis::new().await;

    redis.reset().await;

    let repository = RedisIdempotencyRepository::new(redis.client);

    let key = Uuid::new_v4();
    repository.reserve(key).await.unwrap();

    let response = StoredResponse {
        payment_id: Uuid::new_v4(),
        status: PaymentStatus::Success,
    };

    repository.complete(key, response.clone()).await.unwrap();

    let result = repository.reserve(key).await.unwrap();

    match result {
        ReservationResult::Completed(stored) => {
            assert_eq!(stored, response);
        }

        _ => panic!("Expected completed response"),
    }
}
