use actix_web::{HttpResponse, ResponseError};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum CustomError {
    #[error("Database Connection Error: {0}")]
    DbConnectionError(String),

    #[error("Blocking Error: {0}")]
    BlockingError(String),

    #[error("Query Error: {0}")]
    QueryError(String),

    #[error("Hashing Error: {0}")]
    HashingError(String),

    #[error("Validation Error: {0}")]
    ValidationError(String),

    #[error("Authentication Error: {0}")]
    AuthenticationError(String),
}

impl ResponseError for CustomError {
    fn error_response(&self) -> HttpResponse {
        match self {
            CustomError::QueryError(_) => {
                HttpResponse::InternalServerError().body(self.to_string())
            }
            CustomError::BlockingError(_) => {
                HttpResponse::InternalServerError().body(self.to_string())
            }
            CustomError::DbConnectionError(_) => {
                HttpResponse::InternalServerError().body(self.to_string())
            }
            CustomError::ValidationError(_) => HttpResponse::BadRequest().body(self.to_string()),
            CustomError::HashingError(_) => {
                HttpResponse::InternalServerError().body(self.to_string())
            }
            CustomError::AuthenticationError(_) => {
                HttpResponse::Unauthorized().body(self.to_string())
            }
        }
    }
}
