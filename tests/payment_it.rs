mod common;
use chrono::Utc;
use flux_pay::{
    domain::payment::{repository::PaymentRepository, status::PaymentStatus},
    infrastructure::repositories::postgres_payment_repository::PostgresPaymentRepository,
};

use crate::common::{fixtures::test_payment, test_database::TestDb};

pub struct TestContex {
    pub db: TestDb,
    pub repository: PostgresPaymentRepository,
}

impl TestContex {
    pub async fn new() -> Self {
        let db = TestDb::new().await;
        TestDb::init(&db.pool.clone()).await;
        db.reset().await;

        let repository = PostgresPaymentRepository::new(db.pool.clone());

        Self { db, repository }
    }
}

#[tokio::test]
async fn should_save_and_fetch_payments() {
    let ctx = TestContex::new().await;

    let payment = test_payment();
    ctx.repository.save(&payment).await.unwrap();

    let paid = ctx.repository.find_by_id(payment.id()).await.unwrap();
    assert!(paid.is_some());

    let fetched = paid.unwrap();
    assert_eq!(fetched.merchant_id(), payment.merchant_id());
    assert_eq!(fetched.reference(), payment.reference());
}

#[tokio::test]
async fn should_fetch_payment_using_reference() {
    let ctx = TestContex::new().await;
    let payment = test_payment();

    ctx.repository.save(&payment).await.unwrap();
    let paid = ctx
        .repository
        .find_by_reference(&payment.reference())
        .await
        .unwrap();

    assert!(paid.is_some());

    let fetched = paid.unwrap();

    assert_eq!(fetched.reference(), payment.reference());
}

#[tokio::test]
async fn should_find_all_payments() {
    let ctx = TestContex::new().await;

    for _ in 0..5 {
        let payment = test_payment();
     ctx.repository.save(&payment).await.unwrap();
    }

    let payments = ctx.repository.find_all().await.unwrap();
    assert_eq!(payments.iter().len(), 5);
}

#[tokio::test]
async fn should_update_payment() {
    let ctx = TestContex::new().await;
    let mut payment = test_payment();
    let now = Utc::now();

    ctx.repository.save(&payment).await.unwrap();

    payment.set_status(PaymentStatus::Processing);
    payment.set_provider_reference(Some("provider_reference".to_string()));
    payment.set_failure_reason(Some("failure_reason".to_string()));
    payment.set_paid_at(Some(now));

    ctx.repository.update(&payment).await.unwrap();

    let updated = ctx
        .repository
        .find_by_id(payment.id())
        .await
        .unwrap()
        .unwrap();

    assert_eq!(updated.status(), payment.status());
    assert_eq!(
        updated.provider_reference(),
        Some("provider_reference".to_string())
    );
    assert_eq!(updated.failure_reason(), Some("failure_reason".to_string()))
}

#[tokio::test]
async fn should_delete_payment() {
    let ctx = TestContex::new().await;
    let payment = test_payment();

    ctx.repository.save(&payment).await.unwrap();

    let deleted_rows = ctx.repository.delete(payment.id()).await.unwrap();
    assert_eq!(deleted_rows, 1);
}
