use std::sync::Arc;

use uuid::Uuid;

use crate::{
    domain::{
        errors::domain_error::DomainError,
        idempotency::{
            models::{ReservationResult, StoredResponse},
            repository::IdempotencyRepository,
        },
        orchestration::gateway::PaymentGateway,
        payment::{payment::Payment, repository::PaymentRepository, status::PaymentStatus},
        refund::{
            refund::{Refund, RefundCreationCommand},
            repository::RefundRepository,
        },
    },
    infrastructure::orchestration::provider_registry::ProviderRegistry,
};

pub struct RefundService {
    payments: Arc<dyn PaymentRepository>,
    refunds: Arc<dyn RefundRepository>,
    idempotency: Arc<dyn IdempotencyRepository>,
    providers: Arc<ProviderRegistry>,
}

impl RefundService {
    pub async fn create_refund(&self, cmd: RefundCreationCommand) -> Result<Refund, DomainError> {
        // 1. Domain validation (read-only, nothing to clean up on failure)
        let (payment, gateway) = self.validate(&cmd).await?;

        // 2. Idempotency check
        let key = cmd.idempotency_key;

        match self.idempotency.reserve(key).await? {
            ReservationResult::InProgress => return Err(DomainError::RequestInProgress),
            ReservationResult::Completed(stored) => {
                let refund_id = Uuid::parse_str(&stored.provider_reference)
                    .map_err(|_| DomainError::InvalidStoredResponse)?;
                return self
                    .refunds
                    .find_by_id(refund_id)
                    .await?
                    .ok_or(DomainError::EntityNotFound);
            }
            ReservationResult::Reserved => {}
        }

        // 3. Create refund record (repo checks the refundable balance atomically)
        let refund = Refund::create_refund(payment.id(), cmd.amount, cmd.reason)?;
        let mut refund = self.refunds.create_refund(&refund).await?;

        // 4. Mark Processing BEFORE calling the provider, then call it
        refund.mark_processing()?;
        let refund = self.refunds.update_refund(&refund).await?;

        let refund = match gateway.refund_payment(&refund).await {
            Ok(result) => result,
            Err(e) => {
                // Outcome unknown: stay Processing for reconciliation
                tracing::error!(refund_id = %refund.id(), error = ?e, "provider refund call errored");
                refund
            }
        };

        // 5. Persist final status
        let refund = self.refunds.update_refund(&refund).await?;

        // 6. Complete idempotency (refund id carried in provider_reference)
        let stored = StoredResponse {
            payment_id: refund.payment_id(),
            provider_reference: refund.id().to_string(),
            authorization_url: None,
            client_secret: None,
            status: PaymentStatus::Processing, // placeholder, ignored on replay
        };

        if let Err(e) = self.idempotency.complete(key, stored).await {
            tracing::warn!(error = ?e, "failed to store idempotent response");
        }

        Ok(refund)
    }

    async fn validate(
        &self,
        cmd: &RefundCreationCommand,
    ) -> Result<(Payment, Arc<dyn PaymentGateway>), DomainError> {
        let payment = self
            .payments
            .find_by_id(cmd.payment_id)
            .await?
            .ok_or(DomainError::PaymentNotFound(cmd.payment_id))?;

        let mut problems = Vec::new();
        if !matches!(payment.status(), PaymentStatus::Success) {
            problems.push("only successful payments can be refunded".to_string());
        }
        if cmd.amount.currency() != payment.amount().currency() {
            problems.push("refund currency must match payment currency".to_string());
        }
        if !problems.is_empty() {
            return Err(DomainError::ValidationError(problems));
        }

        let provider = payment.provider().ok_or(DomainError::NoProviderAvailable)?;
        let gateway = self
            .providers
            .get(&provider)
            .ok_or(DomainError::ProviderNotFound(provider))?;

        Ok((payment, gateway))
    }
}
