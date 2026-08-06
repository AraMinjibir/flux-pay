use actix_web::{
    HttpResponse,
    web::{self, Json},
};
use uuid::Uuid;

use crate::{
    api::{
        dto::{PaymentCreationRequest, PaymentCreationResponse, PaymentResponse},
        errors::api_error::ApiError,
    },
    config::app_state::AppState,
    domain::payment::{method::PaymentMethod, provider::PaymentProvider, status::PaymentStatus},
};

pub async fn generate_payment(
    state: web::Data<AppState>,
    payload: Json<PaymentCreationRequest>,
) -> Result<HttpResponse, ApiError> {
    let command = payload.into_inner().into_domain()?;

    let result = state.payment_service.generate_payment(command).await?;

    Ok(HttpResponse::Created().json(PaymentCreationResponse::from_domain(result)))
}

pub async fn find_payment_by_id(
    state: web::Data<AppState>,
    id: web::Path<Uuid>,
) -> Result<HttpResponse, ApiError> {
    let id = id.into_inner();

    let payment = state.payment_service.find_payment_by_id(id).await?;
    Ok(HttpResponse::Ok().json(PaymentResponse::from_domain(&payment)))
}

pub async fn find_payment_by_reference(
    state: web::Data<AppState>,
    reference: web::Path<String>,
) -> Result<HttpResponse, ApiError> {
    let reference = reference.into_inner();

    let paid = state
        .payment_service
        .find_payment_by_reference(&reference)
        .await?;

    Ok(HttpResponse::Ok().json(PaymentResponse::from_domain(&paid)))
}

pub async fn find_payment_by_status(
    state: web::Data<AppState>,
    status: web::Path<PaymentStatus>,
) -> Result<HttpResponse, ApiError> {
    let status = status.into_inner();
    let payments = state
        .payment_service
        .find_payment_by_status(&status)
        .await?;

    let response: Vec<PaymentResponse> =
        payments.iter().map(PaymentResponse::from_domain).collect();

    Ok(HttpResponse::Ok().json(response))
}

pub async fn find_payment_by_provider(
    state: web::Data<AppState>,
    provider: web::Path<PaymentProvider>,
) -> Result<HttpResponse, ApiError> {
    let provider = provider.into_inner();
    let payments = state.payment_service.find_by_provider(&provider).await?;

    let response: Vec<PaymentResponse> =
        payments.iter().map(PaymentResponse::from_domain).collect();

    Ok(HttpResponse::Ok().json(response))
}

pub async fn find_payment_by_method(
    state: web::Data<AppState>,
    method: web::Path<PaymentMethod>,
) -> Result<HttpResponse, ApiError> {
    let method = method.into_inner();
    let payments = state
        .payment_service
        .find_payment_by_method(&method)
        .await?;

    let response: Vec<PaymentResponse> =
        payments.iter().map(PaymentResponse::from_domain).collect();

    Ok(HttpResponse::Ok().json(response))
}

pub async fn find_payment_by_merchant(
    state: web::Data<AppState>,
    id: web::Path<Uuid>,
) -> Result<HttpResponse, ApiError> {
    let id = id.into_inner();
    let payments = state.payment_service.find_payment_by_merchant(id).await?;

    let response: Vec<PaymentResponse> =
        payments.iter().map(PaymentResponse::from_domain).collect();

    Ok(HttpResponse::Ok().json(response))
}

pub async fn find_all_payments(state: web::Data<AppState>) -> Result<HttpResponse, ApiError> {
    let payments = state.payment_service.find_all_payments().await?;

    let response: Vec<PaymentResponse> =
        payments.iter().map(PaymentResponse::from_domain).collect();

    Ok(HttpResponse::Ok().json(response))
}

pub async fn delete_payment(
    state: web::Data<AppState>,
    id: web::Path<Uuid>,
) -> Result<HttpResponse, ApiError> {
    let id = id.into_inner();
    state.payment_service.delete(id).await?;

    Ok(HttpResponse::NoContent().into())
}
