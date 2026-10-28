mod common;
use std::sync::Arc;

use flux_pay::{
    domain::{
        errors::repository_error::RepositoryError, payment::repository::PaymentRepository,
        refund::repository::RefundRepository,
    },
    infrastructure::repositories::{
        postgres_payment_repository::PostgresPaymentRepository,
        postgres_refund_repository::PostgresRefundRepository,
    },
};

use tokio::task::JoinSet;
use uuid::Uuid;

use crate::common::{
    fixtures::{test_payment, test_refund},
    test_database::TestDb,
};

pub struct TestContex {
    pub db: TestDb,
    pub payment_repository: PostgresPaymentRepository,
    pub refund_repository: PostgresRefundRepository,
}

impl TestContex {
    pub async fn new() -> Self {
        let db = TestDb::new().await;
        TestDb::init(&db.pool.clone()).await;
        db.reset().await;

        let payment_repository = PostgresPaymentRepository::new(db.pool.clone());
        let refund_repository = PostgresRefundRepository::new(db.pool.clone());

        Self {
            db,
            payment_repository,
            refund_repository,
        }
    }

    async fn seed_payment(&self, amount: i64) -> Uuid {
        let payment = test_payment();
        self.payment_repository.save(&payment).await.unwrap();

        sqlx::query("UPDATE payments SET amount = $1 WHERE id = $2")
            .bind(amount)
            .bind(payment.id())
            .execute(&self.db.pool)
            .await
            .unwrap();

        payment.id()
    }

    async fn seed_refund_row(&self, payment_id: Uuid, amount: i64, status: &str) {
        let id = Uuid::new_v4();

        sqlx::query(
            r#"
            INSERT INTO refunds (
                id, payment_id, amount, currency, status, reason,
                failure_reason, provider_refund_id, idempotency_key,
                created_at, updated_at
            )
            VALUES ($1, $2, $3, 'NGN', $5, 'seeded', NULL, NULL, $4, NOW(), NOW())
            "#,
        )
        .bind(id)
        .bind(payment_id)
        .bind(amount)
        .bind(format!("seed-{id}"))
        .bind(status)
        .execute(&self.db.pool)
        .await
        .unwrap();
    }

    async fn count_refunds(&self, payment_id: Uuid) -> i64 {
        sqlx::query_scalar("SELECT COUNT(*)::BIGINT FROM refunds WHERE payment_id = $1")
            .bind(payment_id)
            .fetch_one(&self.db.pool)
            .await
            .unwrap()
    }

    async fn sum_refunds(&self, payment_id: Uuid) -> i64 {
        sqlx::query_scalar(
            "SELECT COALESCE(SUM(amount), 0)::BIGINT FROM refunds WHERE payment_id = $1",
        )
        .bind(payment_id)
        .fetch_one(&self.db.pool)
        .await
        .unwrap()
    }
}

#[tokio::test]
async fn should_create_refund_when_amount_is_within_available() {
    let ctx = TestContex::new().await;
    let payment_id = ctx.seed_payment(10_000).await;
    let refund = test_refund(payment_id, 4_000, "idem-happy-1");

    let created = ctx.refund_repository.create_refund(&refund).await.unwrap();

    assert_eq!(created.id(), refund.id());
    assert_eq!(created.payment_id(), payment_id);
    assert_eq!(created.amount().amount(), 4_000);
    assert_eq!(created.status(), refund.status());
    assert_eq!(created.reason(), refund.reason());
    assert_eq!(created.idempotency_key(), refund.idempotency_key());
}

#[tokio::test]
async fn should_persist_refund_in_database() {
    let ctx = TestContex::new().await;
    let payment_id = ctx.seed_payment(10_000).await;
    let refund = test_refund(payment_id, 2_500, "idem-persist-1");

    ctx.refund_repository.create_refund(&refund).await.unwrap();

    let (amount, key): (i64, String) =
        sqlx::query_as("SELECT amount, idempotency_key FROM refunds WHERE id = $1")
            .bind(refund.id())
            .fetch_one(&ctx.db.pool)
            .await
            .unwrap();

    assert_eq!(amount, 2_500);
    assert_eq!(key, "idem-persist-1");
}

#[tokio::test]
async fn should_allow_refund_equal_to_full_payment_amount() {
    let ctx = TestContex::new().await;
    let payment_id = ctx.seed_payment(10_000).await;
    let refund = test_refund(payment_id, 10_000, "idem-full-1");

    let created = ctx.refund_repository.create_refund(&refund).await.unwrap();

    assert_eq!(created.amount().amount(), 10_000);
    assert_eq!(ctx.sum_refunds(payment_id).await, 10_000);
}

#[tokio::test]
async fn should_allow_multiple_partial_refunds_summing_to_payment_amount() {
    let ctx = TestContex::new().await;
    let payment_id = ctx.seed_payment(10_000).await;

    for (i, amount) in [3_000, 3_000, 4_000].into_iter().enumerate() {
        let refund = test_refund(payment_id, amount, &format!("idem-partial-{i}"));
        ctx.refund_repository.create_refund(&refund).await.unwrap();
    }

    assert_eq!(ctx.count_refunds(payment_id).await, 3);
    assert_eq!(ctx.sum_refunds(payment_id).await, 10_000);
}

#[tokio::test]
async fn should_allow_refund_exactly_equal_to_remaining_available() {
    let ctx = TestContex::new().await;
    let payment_id = ctx.seed_payment(10_000).await;
    ctx.seed_refund_row(payment_id, 6_000, "SUCCEEDED").await;

    let refund = test_refund(payment_id, 4_000, "idem-boundary-1");

    ctx.refund_repository.create_refund(&refund).await.unwrap();

    assert_eq!(ctx.sum_refunds(payment_id).await, 10_000);
}

#[tokio::test]
async fn should_reject_refund_exceeding_payment_amount() {
    let ctx = TestContex::new().await;
    let payment_id = ctx.seed_payment(10_000).await;
    let refund = test_refund(payment_id, 10_001, "idem-exceed-1");

    let error = ctx
        .refund_repository
        .create_refund(&refund)
        .await
        .unwrap_err();

    assert!(matches!(
        error,
        RepositoryError::RefundAmountExceedsAvailable {
            requested: 10_001,
            available: 10_000,
        }
    ));
    assert_eq!(ctx.count_refunds(payment_id).await, 0);
}

#[tokio::test]
async fn should_reject_refund_when_active_refunds_consume_the_balance() {
    let ctx = TestContex::new().await;

    // PENDING, PROCESSING and SUCCEEDED all reserve funds.
    for status in ["PENDING", "PROCESSING", "SUCCEEDED"] {
        ctx.db.reset().await;

        let payment_id = ctx.seed_payment(10_000).await;
        ctx.seed_refund_row(payment_id, 7_000, status).await;

        let refund = test_refund(payment_id, 3_001, &format!("idem-reserved-{status}"));

        let error = ctx
            .refund_repository
            .create_refund(&refund)
            .await
            .unwrap_err();

        assert!(
            matches!(
                error,
                RepositoryError::RefundAmountExceedsAvailable {
                    requested: 3_001,
                    available: 3_000,
                }
            ),
            "status {status} should reserve funds, got {error:?}"
        );
        assert_eq!(
            ctx.count_refunds(payment_id).await,
            1,
            "nothing new should be inserted for status {status}"
        );
    }
}

#[tokio::test]
async fn should_ignore_failed_refunds_when_calculating_available_amount() {
    let ctx = TestContex::new().await;
    let payment_id = ctx.seed_payment(10_000).await;
    ctx.seed_refund_row(payment_id, 8_000, "FAILED").await;

    let refund = test_refund(payment_id, 10_000, "idem-ignore-failed-1");

    let created = ctx.refund_repository.create_refund(&refund).await.unwrap();

    assert_eq!(created.amount().amount(), 10_000);
}

#[tokio::test]
async fn should_fail_when_payment_does_not_exist() {
    let ctx = TestContex::new().await;
    let refund = test_refund(Uuid::new_v4(), 1_000, "idem-no-payment-1");

    let result = ctx.refund_repository.create_refund(&refund).await;

    assert!(matches!(result, Err(RepositoryError::DatabaseError(_))));
}

#[tokio::test]
async fn should_fail_on_duplicate_idempotency_key_and_keep_first_refund() {
    let ctx = TestContex::new().await;
    let payment_id = ctx.seed_payment(10_000).await;

    let first = test_refund(payment_id, 1_000, "idem-dup-1");
    let second = test_refund(payment_id, 2_000, "idem-dup-1");

    ctx.refund_repository.create_refund(&first).await.unwrap();
    let result = ctx.refund_repository.create_refund(&second).await;

    assert!(matches!(result, Err(RepositoryError::DuplicateEntity)));

    assert_eq!(ctx.count_refunds(payment_id).await, 1);
    assert_eq!(ctx.sum_refunds(payment_id).await, 1_000);
}

#[tokio::test]
async fn should_never_over_refund_under_concurrent_requests() {
    let ctx = TestContex::new().await;
    let payment_id = ctx.seed_payment(10_000).await;

    let repository = Arc::new(PostgresRefundRepository::new(ctx.db.pool.clone()));

    // 5 x 3_000 = 15_000 requested against 10_000 -> exactly 3 can succeed.
    let mut tasks = JoinSet::new();
    for i in 0..5 {
        let repository = Arc::clone(&repository);
        tasks.spawn(async move {
            let refund = test_refund(payment_id, 3_000, &format!("idem-concurrent-{i}"));
            repository.create_refund(&refund).await
        });
    }

    let mut succeeded = 0;
    let mut rejected = 0;

    while let Some(joined) = tasks.join_next().await {
        match joined.unwrap() {
            Ok(_) => succeeded += 1,
            Err(RepositoryError::RefundAmountExceedsAvailable { .. }) => rejected += 1,
            Err(other) => panic!("unexpected error: {other:?}"),
        }
    }

    assert_eq!(succeeded, 3);
    assert_eq!(rejected, 2);
    assert_eq!(ctx.count_refunds(payment_id).await, 3);
    assert_eq!(ctx.sum_refunds(payment_id).await, 9_000);
}
