mod common;
use std::sync::{Arc, atomic::Ordering};

use crate::common::{
    fixtures::test_payload,
    test_app::{RetryExhaustedRoutingStrategy, spawn_app, spawn_app_with_routing},
};

#[tokio::test]
async fn payment_creation_succeeds() {
    let app = spawn_app().await;

    let payload = test_payload();

    let response = app.create_payment(&payload).await.unwrap();
    let status = response.status();

    response.text().await.unwrap();

    assert_eq!(status, 201);
}

#[tokio::test]
async fn duplicate_payment_request_returns_original_payment() {
    let app = spawn_app().await;

    let payload = test_payload();

    // First request
    let first_response = app
        .client
        .post(format!("{}/api/v1/payments", app.address))
        .json(&payload)
        .send()
        .await
        .unwrap();

    assert_eq!(first_response.status(), 201);

    let first_body: serde_json::Value = first_response.json().await.unwrap();

    // Second request with the SAME idempotency key
    let second_response = app
        .client
        .post(format!("{}/api/v1/payments", app.address))
        .json(&payload)
        .send()
        .await
        .unwrap();

    assert_eq!(second_response.status(), 201);

    let second_body: serde_json::Value = second_response.json().await.unwrap();

    // Both requests must return the same payment.
    assert_eq!(
        first_body["id"], second_body["id"],
        "duplicate request created a different payment"
    );
}

#[tokio::test]
async fn provider_timeout_is_retried() {
    let app = spawn_app().await;

    let payload = test_payload();
    let response = app.create_payment(&payload).await.unwrap();

    response.text().await.unwrap();

    println!(
        "timeout provider requests: {}",
        app.provider.requests.load(Ordering::SeqCst)
    );

    // assert_eq!(response.status(), 500);

    assert_eq!(app.provider.requests.load(Ordering::SeqCst), 3);
}

#[tokio::test]
async fn maximum_retries_exceeded_returns_500() {
    let app = spawn_app_with_routing(Arc::new(RetryExhaustedRoutingStrategy)).await;

    let payload = test_payload();

    let response = app.create_payment(&payload).await.expect("request failed");

    let status = response.status();
    response.text().await.expect("failed to read response body");

    // Provider exhausted all retry attempts.
    assert_eq!(status, 500);

    // MAX_RETRIES = 3
    assert_eq!(app.provider.requests.load(Ordering::SeqCst), 3);
}

#[tokio::test]
async fn database_failure_returns_500() {
    let mut app = spawn_app().await;

    // Kill PostgreSQL.
    drop(app.postgres.take());

    let payload = test_payload();

    let response = app.create_payment(&payload).await.expect("request failed");

    let status = response.status();
    response.text().await.unwrap();

    assert_eq!(status, 500);
}

#[tokio::test]
async fn redis_failure_returns_500() {
    let mut app = spawn_app().await;

    // Kill Redis.
    drop(app.redis.take());

    let payload = test_payload();

    let response = app.create_payment(&payload).await.expect("request failed");

    let status = response.status();
    response.text().await.unwrap();

    assert_eq!(status, 500);
}
