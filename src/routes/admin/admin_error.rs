use actix_web::{HttpResponse, ResponseError};
use std::fmt;
use thiserror::Error;

#[derive(Debug)]
pub enum AdminError {
    DbConnectionError(String),
    AuthenticationError(String),
}

impl fmt::Display for AdminError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            AdminError::AuthenticationError(msg) => write!(f, "Authentication Error: {}", msg),
            AdminError::DbConnectionError(msg) => {
                write!(f, "Database Connection Error: {}", msg)
            }
        }
    }
}

impl ResponseError for AdminError {
    fn error_response(&self) -> HttpResponse {
        match self {
            AdminError::AuthenticationError(_) => {
                HttpResponse::Unauthorized().body(self.to_string())
            }
            AdminError::DbConnectionError(_) => {
                HttpResponse::InternalServerError().body(self.to_string())
            }
        }
    }
}
