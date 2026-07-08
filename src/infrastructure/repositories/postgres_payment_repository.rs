use async_trait::async_trait;
use sqlx::{PgPool, query_as};
use uuid::Uuid;

use crate::{
    domain::{
        errors::repository_error::{RepositoryError, map_sqlx_error},
        payment::{
            method::PaymentMethod, payment::Payment, provider::PaymentProvider,
            repository::PaymentRepository, status::PaymentStatus,
        },
    },
    infrastructure::rows::payment_row::PaymentRow,
};

pub struct PostgresPaymentRepository {
    pool: PgPool,
}

impl PostgresPaymentRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]

impl PaymentRepository for PostgresPaymentRepository {
    async fn save(&self, payment: &Payment) -> Result<(), RepositoryError> {
        let row = PaymentRow::from_domain(payment);
        sqlx::query!(
            r#"
            INSERT INTO payments (
            id, merchant_id, amount, currency, description, reference, status, payment_method, 
           payment_provider, provider_reference, failure_reason, retry_count, idempotency_key, created_at, paid_at
            )
            VALUES($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15)
            "#,
            row.id,
            row.merchant_id,
            row.amount,
            row.currency,
            row.description,
            row.reference,
            row.status,
            row.payment_method,
            row.payment_provider,
            row.provider_reference,
            row.failure_reason,
            row.retry_count,
            row.idempotency_key,
            row.created_at,
            row.paid_at
        ).execute(&self.pool)
        .await
        .map_err(map_sqlx_error)?;

        Ok(())
    }

    async fn find_by_id(&self, id: Uuid) -> Result<Option<Payment>, RepositoryError> {
        let row: Option<PaymentRow> = query_as!(PaymentRow, r#"
        SELECT
        id, merchant_id, amount, currency, description, reference, status, payment_method, 
           payment_provider, provider_reference, failure_reason, retry_count, idempotency_key, created_at, paid_at
           FROM payments
           WHERE id = $1
        "#,id )
        .fetch_optional(&self.pool)
        .await
        .map_err(map_sqlx_error)?;

        Ok(row.map(|r| r.into_domain()))
    }
    async fn find_by_reference(&self, reference: &str) -> Result<Option<Payment>, RepositoryError> {
        let row: Option<PaymentRow> = query_as!(PaymentRow, r#"
        SELECT
        id, merchant_id, amount, currency, description, reference, status, payment_method, 
           payment_provider, provider_reference, failure_reason, retry_count, idempotency_key, created_at, paid_at
           FROM payments
           WHERE reference = $1
        "#,reference )
        .fetch_optional(&self.pool)
        .await
        .map_err(map_sqlx_error)?;

        Ok(row.map(|r| r.into_domain()))
    }

    async fn find_by_status(
        &self,
        status: &PaymentStatus,
    ) -> Result<Vec<Payment>, RepositoryError> {
        let rows: Vec<PaymentRow> = sqlx::query_as!(
            PaymentRow,
            r#"
        SELECT * FROM payments
        WHERE status = $1
        "#,
            status.to_string()
        )
        .fetch_all(&self.pool)
        .await
        .map_err(map_sqlx_error)?;

        Ok(rows.into_iter().map(|rs| rs.into_domain()).collect())
    }

    async fn find_by_provider(
        &self,
        provider: PaymentProvider,
    ) -> Result<Vec<Payment>, RepositoryError> {
        let rows: Vec<PaymentRow> = sqlx::query_as!(
            PaymentRow,
            r#"
        SELECT * FROM payments
        WHERE payment_provider = $1
        "#,
            provider.to_string()
        )
        .fetch_all(&self.pool)
        .await
        .map_err(map_sqlx_error)?;

        Ok(rows.into_iter().map(|rs| rs.into_domain()).collect())
    }

    async fn find_by_method(&self, method: PaymentMethod) -> Result<Vec<Payment>, RepositoryError> {
        let rows: Vec<PaymentRow> = sqlx::query_as!(
            PaymentRow,
            r#"
        SELECT * FROM payments
        WHERE payment_method = $1
        "#,
            method.to_string()
        )
        .fetch_all(&self.pool)
        .await
        .map_err(map_sqlx_error)?;

        Ok(rows.into_iter().map(|rs| rs.into_domain()).collect())
    }
    async fn find_by_merchant(&self, merchant_id: Uuid) -> Result<Vec<Payment>, RepositoryError> {
        let rows: Vec<PaymentRow> = sqlx::query_as!(
            PaymentRow,
            r#"
        SELECT * FROM payments
        WHERE merchant_id = $1"#,
            merchant_id
        )
        .fetch_all(&self.pool)
        .await
        .map_err(map_sqlx_error)?;

        Ok(rows.into_iter().map(|rs| rs.into_domain()).collect())
    }

    async fn find_all(&self) -> Result<Vec<Payment>, RepositoryError> {
        let rows: Vec<PaymentRow> = sqlx::query_as!(
            PaymentRow,
            r#"
        SELECT * FROM payments
       ORDER BY paid_at DESC"#,
        )
        .fetch_all(&self.pool)
        .await
        .map_err(map_sqlx_error)?;
        Ok(rows.into_iter().map(|rs| rs.into_domain()).collect())
    }
    async fn update(&self, payment: &Payment) -> Result<(), RepositoryError> {
        let row = PaymentRow::from_domain(payment);
        sqlx::query!(
            r#"
        UPDATE payments
        SET status = $1,
        provider_reference = $2,
        failure_reason = $3,
        paid_at = $4
        WHERE id = $5
        "#,
            row.status,
            row.provider_reference,
            row.failure_reason,
            row.paid_at,
            row.id
        )
        .execute(&self.pool)
        .await
        .map_err(map_sqlx_error)?;

        Ok(())
    }

    async fn delete(&self, id: Uuid) -> Result<u64, RepositoryError> {
        let result = sqlx::query!(
            r#"
        UPDATE payments
        SET status = $1
        WHERE id = $2
        "#,
            "Deleted",
            id
        )
        .execute(&self.pool)
        .await
        .map_err(map_sqlx_error)?;
        Ok(result.rows_affected())
    }
}
