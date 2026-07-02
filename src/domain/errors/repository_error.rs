#[derive(Debug)]
pub enum RepositoryError {
    DuplicateEntity,
    EntityNotFound,

    ForeignKeyViolation,
    NullConstraintViolation,
    CheckConstraintViolation,

    DataTooLong,
    InvalidDataFormat,
    NumericOverflow,

    DeadlockDetected,
    TransactionTimeout,
    SerializationFailure,

    ConnectionError,
    PoolTimeout,

    DatabaseError(String),
}

pub fn map_sqlx_error(err: sqlx::Error) -> RepositoryError {
    match err {
        sqlx::Error::Database(db_err) => {
            if let Some(code) = db_err.code() {
                let msg = db_err.message().to_string();

                match code.as_ref() {
                    "23505" => RepositoryError::DuplicateEntity,
                    "23503" => RepositoryError::ForeignKeyViolation,
                    "23502" => RepositoryError::NullConstraintViolation,
                    "23514" => RepositoryError::CheckConstraintViolation,
                    "22001" => RepositoryError::DataTooLong,
                    "22003" => RepositoryError::NumericOverflow,
                    "40P01" => RepositoryError::DeadlockDetected,
                    "57014" => RepositoryError::TransactionTimeout,
                    "40001" => RepositoryError::SerializationFailure,
                    _ if code.as_ref().starts_with("22") => RepositoryError::InvalidDataFormat,
                    _ => RepositoryError::DatabaseError(msg),
                }
            } else {
                RepositoryError::DatabaseError(db_err.message().to_string())
            }
        }

        _ => RepositoryError::DatabaseError(err.to_string()),
    }
}
