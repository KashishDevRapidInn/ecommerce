use actix_web::{HttpResponse, ResponseError};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum CustomError {
    #[error("Database Error: {0}")]
    DatabaseError(#[from] DbError),

    #[error("Blocking Error: {0}")]
    BlockingError(String),

    #[error("Hashing Error: {0}")]
    HashingError(String),

    #[error("Validation Error: {0}")]
    ValidationError(String),

    #[error("Authentication Error: {0}")]
    AuthenticationError(#[from] AuthError),
}

#[derive(Debug, Error)]
pub enum DbError {
    #[error("Connection Error: {0}")]
    ConnectionError(String),

    #[error("Query Error: {0}")]
    QueryBuilderError(String),

    #[error("Insertion Error: {0}")]
    InsertionError(String),

    #[error("Updation Error: {0}")]
    UpdationError(String),

    #[error("Other Database Error: {0}")]
    Other(String),
}

#[derive(Debug, Error)]
pub enum AuthError {
    #[error("Session Authentication Error: {0}")]
    SessionAuthenticationError(String),

    #[error("JWT Authentication Error: {0}")]
    JwtAuthenticationError(String),

    #[error("Other Authentication Error: {0}")]
    OtherAuthenticationError(String),
}
impl ResponseError for CustomError {
    fn error_response(&self) -> HttpResponse {
        match self {
            CustomError::BlockingError(_) => {
                HttpResponse::InternalServerError().body(self.to_string())
            }
            CustomError::ValidationError(_) => HttpResponse::BadRequest().body(self.to_string()),
            CustomError::HashingError(_) => {
                HttpResponse::InternalServerError().body(self.to_string())
            }
            CustomError::DatabaseError(err) => match err {
                DbError::ConnectionError(_) => {
                    HttpResponse::InternalServerError().body(self.to_string())
                }
                DbError::QueryBuilderError(_) => {
                    HttpResponse::InternalServerError().body(self.to_string())
                }
                DbError::InsertionError(_) => {
                    HttpResponse::InternalServerError().body(self.to_string())
                }
                DbError::UpdationError(_) => {
                    HttpResponse::InternalServerError().body(self.to_string())
                }
                DbError::Other(_) => HttpResponse::InternalServerError().body(self.to_string()),
            },
            CustomError::AuthenticationError(err) => match err {
                AuthError::SessionAuthenticationError(_) => {
                    HttpResponse::InternalServerError().body(self.to_string())
                }
                AuthError::JwtAuthenticationError(_) => {
                    HttpResponse::InternalServerError().body(self.to_string())
                }
                AuthError::OtherAuthenticationError(_) => {
                    HttpResponse::InternalServerError().body(self.to_string())
                }
            },
        }
    }
}

//concurreny vs parallel

//enums in order -> done
//jsonwebtoken claim -> done
//admin api -> done
//spwan for hash -> done
//deadpool -> done
//env-> config -> done
// map all the errors -> done

// Added enums in order schema
// Added more information in JWT claim
// Added fetch_all_orders api for admin
// Added functionality to perform hashing in a seperate thread
// Made db_operations async
// Replaced .env with a config.rs file using config crate
// Added more detailed error handling
