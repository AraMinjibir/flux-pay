use sqlx::PgPool;
use uuid::Uuid;

use crate::domain::{
    errors::repository_error::{RepositoryError, map_sqlx_error},
    ledger::{account_codes, journal::JournalDraft, posting_plan::PostingPlan},
    payment::provider::PaymentProvider,
    shared::currency::Currency,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PostedJournal {
    pub journal_id: Uuid,
    pub replayed: bool,
}

pub struct PostgresLedgerRepository {
    pool: PgPool,
}

impl PostgresLedgerRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn platform_account_id(
        &self,
        code: &str,
        currency: &Currency,
    ) -> Result<Uuid, RepositoryError> {
        let currency = currency.to_string();
        let row = sqlx::query!(
            r#"
            SELECT id
            FROM ledger_accounts
            WHERE code = $1 AND currency = $2 AND owner_type = 'PLATFORM' AND is_active
            "#,
            code,
            currency
        )
        .fetch_optional(&self.pool)
        .await
        .map_err(map_sqlx_error)?;

        row.map(|r| r.id).ok_or(RepositoryError::EntityNotFound)
    }

    /// Gets or creates the provider's receivable subledger account (`1100-<PROVIDER>`).
    pub async fn ensure_provider_receivable(
        &self,
        provider: &PaymentProvider,
        currency: &Currency,
    ) -> Result<Uuid, RepositoryError> {
        let currency = currency.to_string();
        let suffix = provider.to_string().to_uppercase();
        let code = format!("{}-{}", account_codes::PROVIDER_RECEIVABLE, suffix);
        let name = format!("Provider Receivable - {suffix}");

        sqlx::query!(
            r#"
            INSERT INTO ledger_accounts
                (code, name, account_type, normal_balance, currency, owner_type, parent_account_id)
            SELECT $1, $2, 'ASSET', 'DEBIT', $3, 'PROVIDER', control.id
            FROM ledger_accounts control
            WHERE control.code = $4 AND control.currency = $3 AND control.owner_type = 'PLATFORM'
            ON CONFLICT DO NOTHING
            "#,
            code,
            name,
            currency,
            account_codes::PROVIDER_RECEIVABLE
        )
        .execute(&self.pool)
        .await
        .map_err(map_sqlx_error)?;

        self.account_id_by_code(&code, &currency).await
    }

    /// Gets or creates the merchant's payable subledger account (`2000-<merchant id>`).
    pub async fn ensure_merchant_payable(
        &self,
        merchant_id: Uuid,
        currency: &Currency,
    ) -> Result<Uuid, RepositoryError> {
        let currency = currency.to_string();
        let code = format!("{}-{}", account_codes::MERCHANT_PAYABLE, merchant_id);
        let name = format!("Merchant Payable - {merchant_id}");

        sqlx::query!(
            r#"
            INSERT INTO ledger_accounts
                (code, name, account_type, normal_balance, currency, owner_type, owner_id, parent_account_id)
            SELECT $1, $2, 'LIABILITY', 'CREDIT', $3, 'MERCHANT', $4, control.id
            FROM ledger_accounts control
            WHERE control.code = $5 AND control.currency = $3 AND control.owner_type = 'PLATFORM'
            ON CONFLICT DO NOTHING
            "#,
            code,
            name,
            currency,
            merchant_id,
            account_codes::MERCHANT_PAYABLE
        )
        .execute(&self.pool)
        .await
        .map_err(map_sqlx_error)?;

        self.account_id_by_code(&code, &currency).await
    }

    async fn account_id_by_code(
        &self,
        code: &str,
        currency: &str,
    ) -> Result<Uuid, RepositoryError> {
        let row = sqlx::query!(
            r#"SELECT id FROM ledger_accounts WHERE code = $1 AND currency = $2"#,
            code,
            currency
        )
        .fetch_optional(&self.pool)
        .await
        .map_err(map_sqlx_error)?;
        row.map(|r| r.id).ok_or(RepositoryError::EntityNotFound)
    }

    pub async fn post_journal(
        &self,
        draft: &JournalDraft,
        plan: &PostingPlan,
    ) -> Result<PostedJournal, RepositoryError> {
        let mut tx = self.pool.begin().await.map_err(map_sqlx_error)?;

        let source_type = draft.source_type.as_str();
        let event_type = draft.event_type.as_str();
        let currency = draft.currency.to_string();
        let inserted = sqlx::query!(
            r#"
            INSERT INTO journals
                (reference, source_type, source_id, event_type, idempotency_key, description, currency)
            VALUES ($1, $2, $3, $4, $5, $6, $7)
            ON CONFLICT DO NOTHING
            RETURNING id
            "#,
            draft.reference,
            source_type,
            draft.source_id,
            event_type,
            draft.idempotency_key,
            draft.description,
            currency
        )
        .fetch_optional(&mut *tx)
        .await
        .map_err(map_sqlx_error)?;

        let journal_id = match inserted {
            Some(row) => row.id,
            None => {
                let existing = sqlx::query!(
                    r#"
                    SELECT id, source_type, source_id, event_type, status
                    FROM journals
                    WHERE idempotency_key = $1
                    "#,
                    draft.idempotency_key
                )
                .fetch_optional(&mut *tx)
                .await
                .map_err(map_sqlx_error)?;

                return match existing {
                    Some(e)
                        if e.source_type == source_type
                            && e.source_id == draft.source_id
                            && e.event_type == event_type
                            && e.status == "POSTED" =>
                    {
                        Ok(PostedJournal {
                            journal_id: e.id,
                            replayed: true,
                        })
                    }
                    _ => Err(RepositoryError::DuplicateEntity),
                };
            }
        };

        // 2. Entries, one statement.
        sqlx::query!(
            r#"
            INSERT INTO journal_entries (journal_id, account_id, line_number, amount, direction)
            SELECT $1, t.account_id, t.line_number, t.amount, t.direction
            FROM UNNEST($2::uuid[], $3::smallint[], $4::bigint[], $5::text[])
                AS t(account_id, line_number, amount, direction)
            "#,
            journal_id,
            &plan.entry_account_ids,
            &plan.entry_line_numbers,
            &plan.entry_amounts,
            &plan.entry_directions as &[&str]
        )
        .execute(&mut *tx)
        .await
        .map_err(map_sqlx_error)?;

        // 3. Lock the balance rows in id order (ORDER BY puts the sort below the
        //    row lock), so two postings touching the same accounts cannot deadlock.
        let locked = sqlx::query!(
            r#"
            SELECT account_id
            FROM ledger_account_balances
            WHERE account_id = ANY($1)
            ORDER BY account_id
            FOR UPDATE
            "#,
            &plan.balance_account_ids
        )
        .fetch_all(&mut *tx)
        .await
        .map_err(map_sqlx_error)?;

        if locked.len() != plan.balance_account_ids.len() {
            return Err(RepositoryError::EntityNotFound);
        }

        // 4. Projection update.
        let updated = sqlx::query!(
            r#"
            UPDATE ledger_account_balances AS b
            SET total_debits  = b.total_debits  + d.debits,
                total_credits = b.total_credits + d.credits
            FROM UNNEST($1::uuid[], $2::bigint[], $3::bigint[])
                AS d(account_id, debits, credits)
            WHERE b.account_id = d.account_id
            "#,
            &plan.balance_account_ids,
            &plan.balance_debits,
            &plan.balance_credits
        )
        .execute(&mut *tx)
        .await
        .map_err(map_sqlx_error)?;

        if updated.rows_affected() != plan.balance_account_ids.len() as u64 {
            return Err(RepositoryError::EntityNotFound);
        }

        // 5. DRAFT -> POSTED. The journals trigger re-verifies debits == credits.
        sqlx::query!(
            r#"
            UPDATE journals
            SET status = 'POSTED', posted_at = NOW()
            WHERE id = $1
            "#,
            journal_id
        )
        .execute(&mut *tx)
        .await
        .map_err(map_sqlx_error)?;

        tx.commit().await.map_err(map_sqlx_error)?;

        Ok(PostedJournal {
            journal_id,
            replayed: false,
        })
    }
}
