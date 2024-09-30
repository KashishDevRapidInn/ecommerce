use actix_web::{HttpResponse, ResponseError};
use std::fmt;
use thiserror::Error;

#[derive(Debug)]
pub enum OrderError {
    QueryError(String),
    AuthenticationError(String),
    DbConnectionError(String),
}

impl fmt::Display for OrderError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            OrderError::QueryError(msg) => write!(f, "Query Error: {}", msg),
            OrderError::AuthenticationError(msg) => write!(f, "Authentication Error: {}", msg),
            OrderError::DbConnectionError(msg) => {
                write!(f, "Database Connection Error: {}", msg)
            }
        }
    }
}

impl ResponseError for OrderError {
    fn error_response(&self) -> HttpResponse {
        match self {
            OrderError::QueryError(_) => HttpResponse::InternalServerError().body(self.to_string()),
            OrderError::AuthenticationError(_) => {
                HttpResponse::Unauthorized().body(self.to_string())
            }
            OrderError::DbConnectionError(_) => {
                HttpResponse::InternalServerError().body(self.to_string())
            }
        }
    }
}
