use crate::db::PgPool;
use crate::errors::custom::{AuthError, CustomError, DbError};
use crate::routes::customer::customer::LoginCustomerBody;
use crate::schema::customers::dsl::*;
use crate::telemetry::spawn_blocking_with_tracing;
use argon2::{self, Argon2, PasswordHash, PasswordVerifier};
use diesel::prelude::*;
use diesel_async::RunQueryDsl;
use tracing::instrument;
use uuid::Uuid;

#[instrument(name = "Get stored credentials", skip(user_name, pool), fields(username = %user_name))]
async fn get_stored_credentials(
    user_name: &str,
    pool: &PgPool,
) -> Result<(Uuid, String), CustomError> {
    let mut conn = pool
        .get()
        .await
        .map_err(|err| CustomError::DatabaseError(DbError::ConnectionError(err.to_string())))?;

    let row: Result<Option<Vec<(String, Uuid)>>, diesel::result::Error> = customers
        .filter(username.eq(user_name))
        .select((password_hash, id))
        .load::<(String, Uuid)>(&mut conn)
        .await
        .optional();

    let (id_user, expected_hash_password) = match row {
        Ok(Some(vec)) => {
            if let Some((hash_password, id_user)) = vec.into_iter().next() {
                (id_user, hash_password)
            } else {
                return Err(CustomError::AuthenticationError(
                    AuthError::OtherAuthenticationError("Invalid username or password".to_string()),
                ));
            }
        }
        Ok(None) => {
            return Err(CustomError::AuthenticationError(
                AuthError::OtherAuthenticationError("Invalid username or password".to_string()),
            ));
        }
        Err(err) => {
            return Err(CustomError::DatabaseError(DbError::QueryBuilderError(
                err.to_string(),
            )))
        }
    };
    Ok((id_user, expected_hash_password))
}

#[instrument(name = "Verify password", skip(expected_hash, candidate))]
fn verify_password(expected_hash: &str, candidate: String) -> bool {
    let argon2 = Argon2::default();
    let password_hashed = PasswordHash::new(expected_hash).expect("Failed to parse password hash");

    argon2
        .verify_password(candidate.as_bytes(), &password_hashed)
        .is_ok()
}
#[instrument(name = "Validate credentials", skip(req_login, pool), fields(username = %req_login.username))]
pub async fn validate_credentials(
    pool: &PgPool,
    req_login: &LoginCustomerBody,
) -> Result<Uuid, CustomError> {
    let (user_id, stored_password_hash) = get_stored_credentials(&req_login.username, pool)
        .await
        .unwrap();

    let entered_pasword = req_login.password.to_owned();
    let is_valid = spawn_blocking_with_tracing(move || {
        verify_password(&stored_password_hash, entered_pasword)
    })
    .await
    .map_err(|_| CustomError::HashingError("Failed to hash inside spawn".to_string()))?;
    if is_valid {
        return Ok(user_id);
    } else {
        return Err(CustomError::AuthenticationError(
            AuthError::JwtAuthenticationError("Invalid credentials".to_string()),
        ));
    }
}
