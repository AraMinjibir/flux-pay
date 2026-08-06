use std::sync::Arc;

use async_trait::async_trait;
use tracing::info;
use uuid::Uuid;

use crate::application::payment_orchestrator::PaymentOrchestrator;
use crate::domain::errors::domain_error::DomainError;
use crate::domain::idempotency::models::{ReservationResult, StoredResponse};
use crate::domain::payment::payment::{
    CreatePaymentCommand, Payment, PaymentInitializationRequest, PaymentInitializationResult,
};
use crate::domain::payment::{
    method::PaymentMethod, provider::PaymentProvider, status::PaymentStatus,
};
use crate::domain::services::payment_service::PaymentService;
use crate::domain::{
    payment::repository::PaymentRepository, services::idempotency_service::IdempotencyService,
};

pub struct PaymentServiceImpl {
    payment_repository: Arc<dyn PaymentRepository>,
    idempotency_service: Arc<dyn IdempotencyService>,
    payment_orchestrator: Arc<PaymentOrchestrator>,
}
impl PaymentServiceImpl {
    pub fn new(
        payment_repository: Arc<dyn PaymentRepository>,
        idempotency_service: Arc<dyn IdempotencyService>,
        payment_orchestrator: Arc<PaymentOrchestrator>,
    ) -> Self {
        Self {
            payment_repository,
            idempotency_service,
            payment_orchestrator,
        }
    }
}
#[async_trait]
impl PaymentService for PaymentServiceImpl {
    async fn generate_payment(
        &self,
        command: CreatePaymentCommand,
    ) -> Result<PaymentInitializationResult, DomainError> {

        // 1. Idempotency protection
        info!("1. Reserving idempotency key...");
        let operation = self
            .idempotency_service
            .reserve(command.idempotency_key.clone())
            .await?;

        match operation {
            ReservationResult::Completed(response) => {
                return Ok(PaymentInitializationResult::from_stored_response(response));
            }

            ReservationResult::InProgress => {
                return Err(DomainError::RequestInProgress);
            }

            ReservationResult::Reserved => {}
        }
        
        // 2. Create payment
        let mut payment = Payment::generate_payment(
            command.merchant_id,
            command.amount.clone(),
            command.description.clone(),
            command.payment_method.clone(),
        )?;

        // 3. Persist initial state
        self.payment_repository.save(&payment).await?;

        // 4. Move to Processing
        payment.mark_processing()?;

        self.payment_repository.update(&payment).await?;

        // 5. Build orchestration request
        info!("Building request...");
        let request = PaymentInitializationRequest::converted_request(&command, &payment);
        info!(" Built request: {:?}", request);

        // 6. Execute orchestration
        info!("Calling orchestrator...");
        let execution = match self.payment_orchestrator.initialize_payment(&request).await {
            Ok(execution) => execution,

            Err(error) => {
                payment.apply_failure(&error)?;

                info!("Updating payment after orchestration...");
                self.payment_repository.update(&payment).await?;

                return Err(error);
            }
        };
        info!("returned orchestrator: {:?}", execution);

        // 7. Apply successful orchestration result
        payment.apply_initialization(&execution)?;

        // 8. Persist updated payment
        self.payment_repository.update(&payment).await?;

        // 9. Build response directly from orchestration result
        let response = payment.to_initialization_result(&execution.initialization);
        info!("Response: {:?}", response);

        // 10. Complete idempotency
        let stored_response = StoredResponse::from_payment(payment.id(), &response);
        info!("stored_response: {:?}", stored_response);

        self.idempotency_service
            .complete(command.idempotency_key, stored_response)
            .await?;

        // 11. Return response
        Ok(response)
    }

    async fn find_payment_by_id(&self, id: Uuid) -> Result<Payment, DomainError> {
        let paid = self.payment_repository.find_by_id(id).await?;
        match paid {
            Some(paid) => Ok(paid),
            None => Err(DomainError::PaymentNotFound(id)),
        }
    }

    async fn find_payment_by_reference(&self, reference: &str) -> Result<Payment, DomainError> {
        let fetched_payment = self.payment_repository.find_by_reference(reference).await?;

        match fetched_payment {
            Some(fetched_payment) => Ok(fetched_payment),
            None => Err(DomainError::PaymentWithRefNotFound(reference.to_string())),
        }
    }

    async fn find_payment_by_status(
        &self,
        status: &PaymentStatus,
    ) -> Result<Vec<Payment>, DomainError> {
        self.payment_repository
            .find_by_status(status)
            .await
            .map_err(DomainError::from)
    }

    async fn find_by_provider(
        &self,
        provider: &PaymentProvider,
    ) -> Result<Vec<Payment>, DomainError> {
        self.payment_repository
            .find_by_provider(provider)
            .await
            .map_err(DomainError::from)
    }

    async fn find_payment_by_method(
        &self,
        method: &PaymentMethod,
    ) -> Result<Vec<Payment>, DomainError> {
        self.payment_repository
            .find_by_method(method)
            .await
            .map_err(DomainError::from)
    }

    async fn find_payment_by_merchant(
        &self,
        merchant_id: Uuid,
    ) -> Result<Vec<Payment>, DomainError> {
        self.payment_repository
            .find_by_merchant(merchant_id)
            .await
            .map_err(DomainError::from)
    }

    async fn find_all_payments(&self) -> Result<Vec<Payment>, DomainError> {
        self.payment_repository
            .find_all()
            .await
            .map_err(DomainError::from)
    }

    async fn delete(&self, id: Uuid) -> Result<u64, DomainError> {
        let delete_rows = self
            .payment_repository
            .delete(id)
            .await
            .map_err(DomainError::from)?;

        Ok(delete_rows)
    }
}
