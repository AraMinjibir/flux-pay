use async_trait::async_trait;
use uuid::Uuid;

use crate::domain::{
    errors::domain_error::DomainError,
    idempotency::{
        models::{IdempotencyRecord, IdempotencyStatus, ReservationResult, StoredResponse},
        repository::IdempotencyRepository,
    },
};

pub struct RedisIdempotencyRepository {
    client: redis::Client,
}
impl RedisIdempotencyRepository {
    pub fn new(client: redis::Client) -> Self {
        Self { client }
    }
}

const IDEMPOTENCY_TTL_SECS: u64 = 24 * 60 * 60;
#[async_trait]
impl IdempotencyRepository for RedisIdempotencyRepository {
    async fn reserve(&self, key: Uuid) -> Result<ReservationResult, DomainError> {
        // Acquire a connection to Redis
        let mut conn = self.client.get_multiplexed_async_connection().await?;

        // Build the initial Processing record
        let record = IdempotencyRecord {
            status: IdempotencyStatus::Processing,
            response: None,
        };

        // Serialize it to JSON
        let json = serde_json::to_string(&record)?;

        // Try to reserve the key atomically
        let reserved: Option<String> = redis::cmd("SET")
            .arg(key.to_string())
            .arg(json)
            .arg("NX")
            .arg("EX")
            .arg(IDEMPOTENCY_TTL_SECS)
            .query_async(&mut conn)
            .await?;

        // We successfully reserved the key.
        // This request is the first one.
        if reserved.is_some() {
            return Ok(ReservationResult::Reserved);
        }

        // Somebody else already owns the key.
        // Read the existing record.
        let value: Option<String> = redis::cmd("GET")
            .arg(key.to_string())
            .query_async(&mut conn)
            .await?;

        let value = value.ok_or(DomainError::DatabaseError(
            "Idempotency key disappeared while reserving".into(),
        ))?;

        // Deserialize the stored record.
        let record: IdempotencyRecord = serde_json::from_str(&value)?;

        match record.status {
            IdempotencyStatus::Processing => Ok(ReservationResult::InProgress),

            IdempotencyStatus::Completed => {
                let response = record.response.ok_or(DomainError::DatabaseError(
                    "Completed idempotency record missing response".into(),
                ))?;

                Ok(ReservationResult::Completed(response))
            }
        }
    }

    async fn complete(&self, key: Uuid, response: StoredResponse) -> Result<(), DomainError> {
        // Acquire a Redis connection
        let mut conn = self.client.get_multiplexed_async_connection().await?;

        // Build the completed record
        let record = IdempotencyRecord {
            status: IdempotencyStatus::Completed,
            response: Some(response),
        };

        // Serialize it to JSON
        let json = serde_json::to_string(&record)?;

        // Overwrite the existing value and refresh the TTL
        redis::cmd("SET")
            .arg(key.to_string())
            .arg(json)
            .arg("EX")
            .arg(IDEMPOTENCY_TTL_SECS)
            .query_async::<()>(&mut conn)
            .await?;

        Ok(())
    }
}
