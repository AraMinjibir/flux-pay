use actix_web::{
    HttpRequest, HttpResponse,
    web::{self, Json},
};
use uuid::Uuid;

use crate::{
    api::{
        dto::{RefundCreationRequest, RefundResponse},
        errors::api_error::ApiError,
    },
    config::app_state::AppState,
    domain::errors::domain_error::DomainError,
};

pub async fn send_refund_request(
    state: web::Data<AppState>,
    req: HttpRequest,
    payment_id: web::Path<Uuid>,
    payload: Json<RefundCreationRequest>,
) -> Result<HttpResponse, ApiError> {
    let idempotency_key = idempotency_key(&req)?;

    let command = payload
        .into_inner()
        .into_domain(payment_id.into_inner(), idempotency_key)?;

    let refund = state.refund_service.create_refund(command).await?;

    Ok(HttpResponse::Created().json(RefundResponse::from_domain(&refund)))
}

pub async fn get_refund_by_id(
    state: web::Data<AppState>,
    id: web::Path<Uuid>,
) -> Result<HttpResponse, ApiError> {
    let refund = state
        .refund_service
        .find_refund_by_id(id.into_inner())
        .await?;

    Ok(HttpResponse::Ok().json(RefundResponse::from_domain(&refund)))
}

fn idempotency_key(req: &HttpRequest) -> Result<Uuid, DomainError> {
    req.headers()
        .get("Idempotency-Key")
        .and_then(|value| value.to_str().ok())
        .and_then(|value| Uuid::parse_str(value).ok())
        .ok_or_else(|| {
            DomainError::ValidationError(vec![
                "Idempotency-Key header is required and must be a valid UUID".to_string(),
            ])
        })
}
