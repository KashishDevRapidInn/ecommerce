use actix_web::{HttpResponse, ResponseError};
use std::fmt;
use thiserror::Error;

#[derive(Debug)]
pub enum CustomerError {
    DbConnectionError(String),
    HashingError(String),
    ValidationError(String),
    BlockingError(String),
    QueryError(String),
    AuthenticationError(String),
    UserDoesNotExist(String),
}

impl fmt::Display for CustomerError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            CustomerError::ValidationError(msg) => write!(f, "Validation Error: {}", msg),
            CustomerError::HashingError(msg) => write!(f, "Hashing Error: {}", msg),
            CustomerError::QueryError(msg) => write!(f, "Query Error: {}", msg),
            CustomerError::BlockingError(msg) => write!(f, "Blocking Error: {}", msg),
            CustomerError::AuthenticationError(msg) => write!(f, "Authentication Error: {}", msg),
            CustomerError::DbConnectionError(msg) => {
                write!(f, "Database Connection Error: {}", msg)
            }
            CustomerError::UserDoesNotExist(msg) => write!(f, "User Not Found Error: {}", msg),
        }
    }
}

impl ResponseError for CustomerError {
    fn error_response(&self) -> HttpResponse {
        match self {
            CustomerError::ValidationError(_) => HttpResponse::BadRequest().body(self.to_string()),
            CustomerError::HashingError(_) => {
                HttpResponse::InternalServerError().body(self.to_string())
            }
            CustomerError::QueryError(_) => {
                HttpResponse::InternalServerError().body(self.to_string())
            }
            CustomerError::BlockingError(_) => {
                HttpResponse::InternalServerError().body(self.to_string())
            }
            CustomerError::AuthenticationError(_) => {
                HttpResponse::Unauthorized().body(self.to_string())
            }
            CustomerError::DbConnectionError(_) => {
                HttpResponse::InternalServerError().body(self.to_string())
            }
            CustomerError::UserDoesNotExist(_) => HttpResponse::NotFound().body(self.to_string()),
        }
    }
}
