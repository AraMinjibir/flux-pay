mod common;
use flux_pay::{
    domain::{errors::domain_error::DomainError, payment::repository::PaymentRepository},
    infrastructure::repositories::postgres_payment_repository::PostgresPaymentRepository,
};

use crate::common::{
    fixtures::{mock_execution, test_payment},
    test_database::TestDb,
};

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

    for _ in 0.. {
        let payment = test_payment();
        ctx.repository.save(&payment).await.unwrap();
    }

    let payments = ctx.repository.find_all().await.unwrap();
    assert_eq!(payments.iter().len(), 4);
}

#[tokio::test]
async fn should_update_payment() {
    let ctx = TestContex::new().await;
    let mut payment = test_payment();

    ctx.repository.save(&payment).await.unwrap();

    let execution = mock_execution();

    payment.apply_initialization(&execution).unwrap();
    payment.mark_processing().unwrap();

    let error = DomainError::DatabaseError("Connection refused".to_string());

    payment.apply_failure(&error).unwrap();

    ctx.repository.update(&payment).await.unwrap();

    let updated = ctx
        .repository
        .find_by_id(payment.id())
        .await
        .unwrap()
        .unwrap();

    assert_eq!(updated.status(), payment.status());
    assert_eq!(updated.provider_reference(), Some("MOCK-123".to_string()));
}

#[tokio::test]
async fn should_delete_payment() {
    let ctx = TestContex::new().await;
    let payment = test_payment();

    ctx.repository.save(&payment).await.unwrap();

    let deleted_rows = ctx.repository.delete(payment.id()).await.unwrap();
    assert_eq!(deleted_rows, 1);
}
