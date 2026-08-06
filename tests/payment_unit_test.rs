use flux_pay::domain::{
    errors::domain_error::DomainError,
    idempotency::models::{ReservationResult, StoredResponse},
    payment::{
        payment::PaymentInitializationResult, provider::PaymentProvider, status::PaymentStatus,
    },
    services::payment_service::PaymentService,
};
use mockall::Sequence;
use uuid::Uuid;

use crate::common::fixtures::TestContext;

mod common;

#[tokio::test]
async fn generate_payment_returns_cached_response_when_completed() {
    let mut context = TestContext::new();

    context.payment_repo.expect_save().times(0);

    context.payment_repo.expect_update().times(0);

    let stored_response = StoredResponse {
        payment_id: Uuid::new_v4(),
        provider_reference: "REF-123".to_string(),
        authorization_url: Some("https://pay.flux".to_string()),
        client_secret: Some("secret".to_string()),
        status: PaymentStatus::Created,
    };

    context
        .idempotency
        .expect_reserve()
        .times(1)
        .return_once(move |_| Ok(ReservationResult::Completed(stored_response)));

    context.idempotency.expect_complete().times(0);

    context
        .gateway
        .expect_provider()
        .return_const(PaymentProvider::Mock);

    context.routing.expect_select().times(0);

    let service = context.build_service();

    let command = TestContext::command();

    let result = service.generate_payment(command).await;

    assert!(result.is_ok());

    let response = result.unwrap();

    assert_eq!(response.provider_reference, "REF-123");

    assert_eq!(
        response.authorization_url,
        Some("https://pay.flux".to_string())
    );

    assert_eq!(response.client_secret, Some("secret".to_string()));

    assert_eq!(response.status, PaymentStatus::Created);
}

#[tokio::test]
async fn generate_payment_returns_request_in_progress() {
    let mut context = TestContext::new();

    context.payment_repo.expect_save().times(0);

    context.payment_repo.expect_update().times(0);

    context
        .idempotency
        .expect_reserve()
        .times(1)
        .return_once(|_| Ok(ReservationResult::InProgress));

    context.idempotency.expect_complete().times(0);

    // Dummy gateway

    context
        .gateway
        .expect_provider()
        .return_const(PaymentProvider::Mock);

    context.routing.expect_select().times(0);

    // Service

    let service = context.build_service();

    let command = TestContext::command();

    let result = service.generate_payment(command).await;

    assert!(result.is_err());

    match result.unwrap_err() {
        DomainError::RequestInProgress => {}

        error => panic!("unexpected error: {:?}", error),
    }
}

#[tokio::test]
async fn generate_payment_succeeds_when_reserved() {
    let mut context = TestContext::new();

    context
        .gateway
        .expect_provider()
        .times(1)
        .return_const(PaymentProvider::Mock);

    context
        .gateway
        .expect_initialize_payment()
        .times(1)
        .return_once(|_| {
            Ok(PaymentInitializationResult {
                provider_reference: "REF-123".to_string(),
                selected_provider: None,
                authorization_url: Some("https://pay.flux".to_string()),
                client_secret: Some("secret".to_string()),
                status: PaymentStatus::Created,
                amount: None,
                created_at: None,
                id: None,
                merchant_id: None,
                reference: None,
                description: None,
            })
        });

    context
        .routing
        .expect_select()
        .times(1)
        .return_once(|_| vec![PaymentProvider::Mock]);

    context
        .idempotency
        .expect_reserve()
        .times(1)
        .return_once(|_| Ok(ReservationResult::Reserved));

    context
        .idempotency
        .expect_complete()
        .times(1)
        .return_once(|_, _| Ok(()));

    context
        .payment_repo
        .expect_save()
        .times(1)
        .return_once(|_| Ok(()));

    context
        .payment_repo
        .expect_update()
        .times(2)
        .returning(|_| Ok(()));

    let command = TestContext::command();

    // Service

    let service = context.build_service();

    let result = service.generate_payment(command).await;

    assert!(result.is_ok());

    let response = result.unwrap();

    assert_eq!(response.provider_reference, "REF-123");

    assert_eq!(
        response.authorization_url,
        Some("https://pay.flux".to_string())
    );

    assert_eq!(response.client_secret, Some("secret".to_string()));
}

#[tokio::test]
async fn generate_payment_fails_when_provider_fails() {
    let mut context = TestContext::new();

    context
        .gateway
        .expect_provider()
        .times(1)
        .return_const(PaymentProvider::Mock);

    context
        .gateway
        .expect_initialize_payment()
        .times(1)
        .return_once(|_| {
            Err(DomainError::ValidationError(vec![
                "invalid amount".to_string(),
            ]))
        });

    context
        .routing
        .expect_select()
        .times(1)
        .return_once(|_| vec![PaymentProvider::Mock]);

    context
        .idempotency
        .expect_reserve()
        .times(1)
        .return_once(|_| Ok(ReservationResult::Reserved));

    context.idempotency.expect_complete().times(0);

    context
        .payment_repo
        .expect_save()
        .times(1)
        .return_once(|_| Ok(()));

    // processing update + failure update
    context
        .payment_repo
        .expect_update()
        .times(2)
        .returning(|_| Ok(()));

    let service = context.build_service();

    let command = TestContext::command();

    let result = service.generate_payment(command).await;

    assert!(matches!(
        result,
        Err(DomainError::PaymentProviderFailed { .. })
    ));
}

#[tokio::test]
async fn generate_payment_retries_and_succeeds() {
    let mut context = TestContext::new();

    let mut sequence = Sequence::new();

    context
        .gateway
        .expect_provider()
        .times(1)
        .return_const(PaymentProvider::Mock);

    context
        .gateway
        .expect_initialize_payment()
        .times(1)
        .in_sequence(&mut sequence)
        .return_once(|_| Err(DomainError::ReqwestError("temporary failure".to_string())));

    context
        .gateway
        .expect_initialize_payment()
        .times(1)
        .in_sequence(&mut sequence)
        .return_once(|_| {
            Ok(PaymentInitializationResult {
                provider_reference: "REF-123".to_string(),
                selected_provider: None,
                authorization_url: Some("https://pay.flux".to_string()),
                client_secret: Some("secret".to_string()),
                status: PaymentStatus::Created,
                amount: None,
                created_at: None,
                id: None,
                merchant_id: None,
                reference: None,
                description: None,
            })
        });

    context
        .idempotency
        .expect_reserve()
        .times(1)
        .return_once(|_| Ok(ReservationResult::Reserved));

    context
        .idempotency
        .expect_complete()
        .times(1)
        .return_once(|_, _| Ok(()));

    context
        .payment_repo
        .expect_save()
        .times(1)
        .return_once(|_| Ok(()));

    context
        .payment_repo
        .expect_update()
        .times(2)
        .returning(|_| Ok(()));

    context
        .routing
        .expect_select()
        .times(1)
        .return_once(|_| vec![PaymentProvider::Mock]);

    let service = context.build_service();

    let command = TestContext::command();

    let result = service.generate_payment(command).await;

    assert!(result.is_ok());

    let response = result.unwrap();

    assert_eq!(response.provider_reference, "REF-123");
}

#[tokio::test]
async fn generate_payment_fails_after_all_retries_are_exhausted() {
    let mut context = TestContext::new();

    context
        .gateway
        .expect_provider()
        .times(1)
        .return_const(PaymentProvider::Mock);

    context
        .gateway
        .expect_initialize_payment()
        .times(3)
        .returning(|_| Err(DomainError::ReqwestError("temporary failure".to_string())));

    context
        .idempotency
        .expect_reserve()
        .times(1)
        .return_once(|_| Ok(ReservationResult::Reserved));

    context.idempotency.expect_complete().times(0);

    context
        .payment_repo
        .expect_save()
        .times(1)
        .return_once(|_| Ok(()));

    context
        .payment_repo
        .expect_update()
        .times(2)
        .returning(|_| Ok(()));

    context
        .routing
        .expect_select()
        .times(1)
        .return_once(|_| vec![PaymentProvider::Mock]);

    let service = context.build_service();
    let command = TestContext::command();

    let result = service.generate_payment(command).await;

    assert!(result.is_err());

    assert!(matches!(
        result,
        Err(DomainError::PaymentProviderFailed { .. })
    ));
}
