use flux_pay::domain::{
    errors::domain_error::DomainError,
    idempotency::models::ReservationResult,
    payment::{provider::PaymentProvider, status::PaymentStatus},
    refund::status::RefundStatus,
    shared::currency::Currency,
};
use mockall::Sequence;
use uuid::Uuid;

use crate::common::fixtures::RefundTestContext;

mod common;

#[tokio::test]
async fn create_refund_fails_when_payment_not_found() {
    let mut context = RefundTestContext::new();

    context
        .payment_repo
        .expect_find_by_id()
        .times(1)
        .return_once(|_| Ok(None));

    context.idempotency.expect_reserve().times(0);
    context.refund_repo.expect_create_refund().times(0);

    let command = RefundTestContext::command();
    let payment_id = command.payment_id;

    let result = context.build_service().create_refund(command).await;

    assert!(matches!(result, Err(DomainError::PaymentNotFound(id)) if id == payment_id));
}

#[tokio::test]
async fn create_refund_fails_when_payment_not_succeeded() {
    let mut context = RefundTestContext::new();

    let payment = RefundTestContext::payment(
        PaymentStatus::Created,
        Currency::NGN,
        Some(PaymentProvider::Mock),
    );

    context
        .payment_repo
        .expect_find_by_id()
        .times(1)
        .return_once(move |_| Ok(Some(payment)));

    context.idempotency.expect_reserve().times(0);
    context.refund_repo.expect_create_refund().times(0);

    let result = context
        .build_service()
        .create_refund(RefundTestContext::command())
        .await;

    match result {
        Err(DomainError::ValidationError(problems)) => assert_eq!(problems.len(), 1),
        other => panic!("unexpected result: {:?}", other),
    }
}

#[tokio::test]
async fn create_refund_fails_on_currency_mismatch() {
    let mut context = RefundTestContext::new();

    let payment = RefundTestContext::payment(
        PaymentStatus::Success,
        Currency::USD, // command is NGN
        Some(PaymentProvider::Mock),
    );

    context
        .payment_repo
        .expect_find_by_id()
        .times(1)
        .return_once(move |_| Ok(Some(payment)));

    context.idempotency.expect_reserve().times(0);
    context.refund_repo.expect_create_refund().times(0);

    let result = context
        .build_service()
        .create_refund(RefundTestContext::command())
        .await;

    match result {
        Err(DomainError::ValidationError(problems)) => assert_eq!(problems.len(), 1),
        other => panic!("unexpected result: {:?}", other),
    }
}

#[tokio::test]
async fn create_refund_returns_both_validation_problems() {
    let mut context = RefundTestContext::new();

    let payment = RefundTestContext::payment(
        PaymentStatus::Created,
        Currency::USD,
        Some(PaymentProvider::Mock),
    );
    context
        .payment_repo
        .expect_find_by_id()
        .times(1)
        .return_once(move |_| Ok(Some(payment)));

    context.idempotency.expect_reserve().times(0);

    let result = context
        .build_service()
        .create_refund(RefundTestContext::command())
        .await;

    match result {
        Err(DomainError::ValidationError(problems)) => assert_eq!(problems.len(), 2),
        other => panic!("unexpected result: {:?}", other),
    }
}

#[tokio::test]
async fn create_refund_fails_when_payment_has_no_provider() {
    let mut context = RefundTestContext::new();

    let payment = RefundTestContext::payment(PaymentStatus::Success, Currency::NGN, None);
    context
        .payment_repo
        .expect_find_by_id()
        .times(1)
        .return_once(move |_| Ok(Some(payment)));

    context.idempotency.expect_reserve().times(0);

    let result = context
        .build_service()
        .create_refund(RefundTestContext::command())
        .await;

    assert!(matches!(result, Err(DomainError::NoProviderAvailable)));
}

#[tokio::test]
async fn create_refund_fails_when_provider_not_registered() {
    let mut context = RefundTestContext::new();

    let payment = RefundTestContext::payment(
        PaymentStatus::Success,
        Currency::NGN,
        Some(PaymentProvider::Mock),
    );

    context
        .payment_repo
        .expect_find_by_id()
        .times(1)
        .return_once(move |_| Ok(Some(payment)));

    context.idempotency.expect_reserve().times(0);

    let result = context
        .build_service_with_provider(PaymentProvider::Paystack)
        .create_refund(RefundTestContext::command())
        .await;

    assert!(matches!(
        result,
        Err(DomainError::ProviderNotFound(PaymentProvider::Mock))
    ));
}

fn expect_valid_payment(context: &mut RefundTestContext) {
    let payment = RefundTestContext::payment(
        PaymentStatus::Success,
        Currency::NGN,
        Some(PaymentProvider::Mock),
    );

    context
        .payment_repo
        .expect_find_by_id()
        .times(1)
        .return_once(move |_| Ok(Some(payment)));
}

#[tokio::test]
async fn create_refund_returns_request_in_progress() {
    let mut context = RefundTestContext::new();
    expect_valid_payment(&mut context);

    context
        .idempotency
        .expect_reserve()
        .times(1)
        .return_once(|_| Ok(ReservationResult::InProgress));

    context.idempotency.expect_complete().times(0);
    context.refund_repo.expect_create_refund().times(0);
    context.gateway.expect_refund_payment().times(0);

    let result = context
        .build_service()
        .create_refund(RefundTestContext::command())
        .await;

    assert!(matches!(result, Err(DomainError::RequestInProgress)));
}

#[tokio::test]
async fn create_refund_returns_existing_refund_when_completed() {
    let mut context = RefundTestContext::new();
    expect_valid_payment(&mut context);

    let existing = RefundTestContext::pending_refund(Uuid::new_v4());
    let existing_id = existing.id();
    let stored = RefundTestContext::stored_response(existing_id.to_string());

    context
        .idempotency
        .expect_reserve()
        .times(1)
        .return_once(move |_| Ok(ReservationResult::Completed(stored)));

    context
        .refund_repo
        .expect_find_by_id()
        .times(1)
        .withf(move |id| *id == existing_id)
        .return_once(move |_| Ok(Some(existing)));

    context.refund_repo.expect_create_refund().times(0);
    context.refund_repo.expect_update_refund().times(0);
    context.gateway.expect_refund_payment().times(0);
    context.idempotency.expect_complete().times(0);

    let result = context
        .build_service()
        .create_refund(RefundTestContext::command())
        .await;

    assert_eq!(result.unwrap().id(), existing_id);
}

#[tokio::test]
async fn create_refund_fails_with_invalid_stored_reference() {
    let mut context = RefundTestContext::new();
    expect_valid_payment(&mut context);

    let stored = RefundTestContext::stored_response("not-a-uuid".to_string());

    context
        .idempotency
        .expect_reserve()
        .times(1)
        .return_once(move |_| Ok(ReservationResult::Completed(stored)));

    context.refund_repo.expect_find_by_id().times(0);
    context.refund_repo.expect_create_refund().times(0);

    let result = context
        .build_service()
        .create_refund(RefundTestContext::command())
        .await;

    assert!(matches!(result, Err(DomainError::InvalidStoredResponse)));
}

#[tokio::test]
async fn create_refund_fails_when_completed_refund_is_missing() {
    let mut context = RefundTestContext::new();
    expect_valid_payment(&mut context);

    let stored = RefundTestContext::stored_response(Uuid::new_v4().to_string());

    context
        .idempotency
        .expect_reserve()
        .times(1)
        .return_once(move |_| Ok(ReservationResult::Completed(stored)));

    context
        .refund_repo
        .expect_find_by_id()
        .times(1)
        .return_once(|_| Ok(None));

    context.refund_repo.expect_create_refund().times(0);

    let result = context
        .build_service()
        .create_refund(RefundTestContext::command())
        .await;

    assert!(matches!(result, Err(DomainError::EntityNotFound)));
}
fn expect_successful_provider_flow(context: &mut RefundTestContext, sequence: &mut Sequence) {
    expect_valid_payment(context);

    context
        .idempotency
        .expect_reserve()
        .times(1)
        .in_sequence(sequence)
        .return_once(|_| Ok(ReservationResult::Reserved));

    context
        .refund_repo
        .expect_create_refund()
        .times(1)
        .in_sequence(sequence)
        .returning(|refund| Ok(refund.clone()));

    context
        .refund_repo
        .expect_update_refund()
        .times(1)
        .in_sequence(sequence)
        .withf(|refund| refund.status() == RefundStatus::Processing)
        .returning(|refund| Ok(refund.clone()));

    context
        .gateway
        .expect_refund_payment()
        .times(1)
        .in_sequence(sequence)
        .returning(|refund| {
            let mut refunded = refund.clone();
            refunded.mark_refunded(Some("MOCK-REFUND-1".to_string()))?;
            Ok(refunded)
        });

    context
        .refund_repo
        .expect_update_refund()
        .times(1)
        .in_sequence(sequence)
        .withf(|refund| refund.status() == RefundStatus::Refunded)
        .returning(|refund| Ok(refund.clone()));
}

#[tokio::test]
async fn create_refund_succeeds_in_the_correct_order() {
    let mut context = RefundTestContext::new();
    let mut sequence = Sequence::new();

    expect_successful_provider_flow(&mut context, &mut sequence);

    context
        .idempotency
        .expect_complete()
        .times(1)
        .in_sequence(&mut sequence)
        .return_once(|_, _| Ok(()));

    let result = context
        .build_service()
        .create_refund(RefundTestContext::command())
        .await;

    assert!(result.is_ok());
}

#[tokio::test]
async fn create_refund_returns_refunded_refund_with_provider_id() {
    let mut context = RefundTestContext::new();
    let mut sequence = Sequence::new();

    expect_successful_provider_flow(&mut context, &mut sequence);

    context
        .idempotency
        .expect_complete()
        .times(1)
        .return_once(|_, _| Ok(()));

    let refund = context
        .build_service()
        .create_refund(RefundTestContext::command())
        .await
        .unwrap();

    assert_eq!(refund.status(), RefundStatus::Refunded);
    assert_eq!(
        refund.provider_refund_id(),
        Some("MOCK-REFUND-1".to_string())
    );
}

#[tokio::test]
async fn create_refund_completes_idempotency_with_refund_id() {
    let mut context = RefundTestContext::new();
    let mut sequence = Sequence::new();

    expect_successful_provider_flow(&mut context, &mut sequence);

    context
        .idempotency
        .expect_complete()
        .times(1)
        .withf(|_, stored| Uuid::parse_str(&stored.provider_reference).is_ok())
        .return_once(|_, _| Ok(()));

    let refund = context
        .build_service()
        .create_refund(RefundTestContext::command())
        .await
        .unwrap();
    assert_eq!(refund.id().to_string().len(), 36);
}

#[tokio::test]
async fn create_refund_still_succeeds_when_complete_fails() {
    let mut context = RefundTestContext::new();
    let mut sequence = Sequence::new();

    expect_successful_provider_flow(&mut context, &mut sequence);

    context
        .idempotency
        .expect_complete()
        .times(1)
        .return_once(|_, _| Err(DomainError::RedisError("connection reset".to_string())));

    let result = context
        .build_service()
        .create_refund(RefundTestContext::command())
        .await;

    assert!(result.is_ok());
}
