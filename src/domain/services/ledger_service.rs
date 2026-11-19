use uuid::Uuid;

use crate::{
    domain::{
        errors::{domain_error::DomainError, ledger_error::LedgerError},
        ledger::{
            account_codes,
            journal_posting_rule::{
                PaymentCapturedInput, ProviderSettledInput, payment_captured, provider_settled,
            },
            posting_plan::PostingPlan,
        },
        payment::provider::PaymentProvider,
        shared::money::Money,
    },
    infrastructure::repositories::postgress_ledger_repository::{
        PostedJournal, PostgresLedgerRepository,
    },
};

/// A payment the provider has confirmed.
pub struct PaymentCaptured {
    pub payment_id: Uuid,
    pub merchant_id: Uuid,
    pub provider: PaymentProvider,
    pub gross: Money,
    /// FluxPay's fee deducted from the merchant. `None` means no fee.
    pub platform_fee: Option<Money>,
}

/// A settlement the provider has paid into FluxPay's bank account.
pub struct ProviderSettled {
    pub settlement_id: Uuid,
    pub provider: PaymentProvider,
    /// Cash that actually arrived.
    pub received: Money,
    /// Fee the provider withheld. `None` means no fee.
    pub provider_fee: Option<Money>,
}

pub struct LedgerService {
    repository: PostgresLedgerRepository,
}

impl LedgerService {
    pub fn new(repository: PostgresLedgerRepository) -> Self {
        Self { repository }
    }

    /// Idempotent per payment: recording the same payment twice returns the
    /// original journal with `replayed: true`.
    pub async fn record_payment_captured(
        &self,
        event: PaymentCaptured,
    ) -> Result<PostedJournal, DomainError> {
        let currency = event.gross.currency().clone();

        let provider_receivable = self
            .repository
            .ensure_provider_receivable(&event.provider, &currency)
            .await?;
        let merchant_payable = self
            .repository
            .ensure_merchant_payable(event.merchant_id, &currency)
            .await?;
        let platform_fee_revenue = self
            .repository
            .platform_account_id(account_codes::PLATFORM_FEE_REVENUE, &currency)
            .await?;

        let draft = payment_captured(PaymentCapturedInput {
            payment_id: event.payment_id,
            reference: format!("PAY-CAPTURED-{}", event.payment_id),
            idempotency_key: format!("payment-captured:{}", event.payment_id),
            gross: event.gross,
            platform_fee: event.platform_fee.ok_or(LedgerError::InvalidAmount)?,
            provider_receivable,
            merchant_payable,
            platform_fee_revenue,
        })?;

        let plan = PostingPlan::from_draft(&draft)?;
        Ok(self.repository.post_journal(&draft, &plan).await?)
    }

    /// Idempotent per settlement.
    pub async fn record_provider_settled(
        &self,
        event: ProviderSettled,
    ) -> Result<PostedJournal, DomainError> {
        let currency = event.received.currency().clone();

        let provider_receivable = self
            .repository
            .ensure_provider_receivable(&event.provider, &currency)
            .await?;
        let settlement_bank = self
            .repository
            .platform_account_id(account_codes::SETTLEMENT_BANK, &currency)
            .await?;
        let provider_fee_expense = self
            .repository
            .platform_account_id(account_codes::PROVIDER_FEE_EXPENSE, &currency)
            .await?;

        let draft = provider_settled(ProviderSettledInput {
            settlement_id: event.settlement_id,
            reference: format!("PROVIDER-SETTLED-{}", event.settlement_id),
            idempotency_key: format!("provider-settled:{}", event.settlement_id),
            received: event.received,
            provider_fee: event.provider_fee.ok_or(LedgerError::InvalidAmount)?,
            provider_receivable,
            settlement_bank,
            provider_fee_expense,
        })?;

        let plan = PostingPlan::from_draft(&draft)?;
        Ok(self.repository.post_journal(&draft, &plan).await?)
    }
}
