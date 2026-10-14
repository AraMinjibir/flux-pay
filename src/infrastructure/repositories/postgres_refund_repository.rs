use async_trait::async_trait;
use sqlx::PgPool;

use crate::domain::{
    errors::repository_error::{RepositoryError, map_sqlx_error},
    refund::{refund::Refund, repository::RefundRepository},
};

use crate::infrastructure::rows::refund_row::RefundRow;
pub struct PostgresRefundRepository {
    pool: PgPool,
}

impl PostgresRefundRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]

impl RefundRepository for PostgresRefundRepository {
    async fn create_refund(&self, refund: &Refund) -> Result<Refund, RepositoryError> {
        let mut tx = self.pool.begin().await.map_err(map_sqlx_error)?;

        // 1. Lock the payment row
        let payment = sqlx::query!(
            r#"
            SELECT amount
            FROM payments
            WHERE id = $1
            FOR UPDATE
            "#,
            refund.payment_id()
        )
        .fetch_one(&mut *tx)
        .await
        .map_err(map_sqlx_error)?;

        // 2. Calculate the amount already reserved for refunds
        let refunded_amount = sqlx::query_scalar!(
            r#"
            SELECT COALESCE(SUM(amount), 0)::BIGINT AS "refunded_amount!"
            FROM refunds
            WHERE payment_id = $1
              AND status IN ('PENDING', 'PROCESSING', 'SUCCEEDED')
            "#,
            refund.payment_id()
        )
        .fetch_one(&mut *tx)
        .await
        .map_err(map_sqlx_error)?;

        // 3. Check how much is still refundable
        let available_amount = payment.amount - refunded_amount;

        if refund.amount().amount() > available_amount {
            return Err(RepositoryError::RefundAmountExceedsAvailable {
                requested: refund.amount().amount(),
                available: available_amount,
            });
        }

        // 4. Create the refund
        let row = RefundRow::from_refund_domain(refund);

        let created_row = sqlx::query_as!(
            RefundRow,
            r#"
    INSERT INTO refunds (
        id,
        payment_id,
        amount,
        currency,
        status,
        reason,
        failure_reason,
        provider_refund_id,
        idempotency_key,
        created_at,
        updated_at
    )
    VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11)
    RETURNING
        id,
        payment_id,
        amount,
        currency,
        status,
        reason,
        failure_reason,
        provider_refund_id,
        idempotency_key,
        created_at,
        updated_at
    "#,
            row.id,
            row.payment_id,
            row.amount,
            row.currency,
            row.status,
            row.reason,
            row.failure_reason,
            row.provider_refund_id,
            row.idempotency_key,
            row.created_at,
            row.updated_at
        )
        .fetch_one(&mut *tx)
        .await
        .map_err(map_sqlx_error)?;

        let created_refund = created_row
            .into_refund_domain()
            .map_err(RepositoryError::from)?;

        // 5. Commit everything atomically
        tx.commit().await.map_err(map_sqlx_error)?;

        Ok(created_refund)
    }
}
