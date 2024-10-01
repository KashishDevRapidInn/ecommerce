use crate::db::PgPool;
use crate::db_models::Customer;
use crate::routes::customer::customer::LoginCustomerBody;
use crate::schema::customers::dsl::*;
use crate::Errors::custom::CustomError;
use argon2::{
    self, password_hash::SaltString, Argon2, PasswordHash, PasswordHasher, PasswordVerifier,
};
use diesel::prelude::*;
use tracing::instrument;
use uuid::Uuid;

#[instrument(name = "Get stored credentials", skip(user_name, pool), fields(username = %user_name))]
async fn get_stored_credentials(
    user_name: &str,
    pool: &PgPool,
) -> Result<(Uuid, String), CustomError> {
    let mut conn = pool.get().expect("Failed to get db connection from pool");

    let row: Result<Option<Vec<(String, Uuid)>>, diesel::result::Error> = customers
        .filter(username.eq(user_name))
        .select((password_hash, id))
        .load::<(String, Uuid)>(&mut conn)
        .optional();

    let (id_user, expected_hash_password) = match row {
        Ok(Some(vec)) => {
            if let Some((hash_password, id_user)) = vec.into_iter().next() {
                (id_user, hash_password)
            } else {
                return Err(CustomError::AuthenticationError(
                    "Invalid username or password".to_string(),
                ));
            }
        }
        Ok(None) => {
            return Err(CustomError::AuthenticationError(
                "Invalid username or password".to_string(),
            ));
        }
        Err(err) => {
            return Err(CustomError::DbConnectionError(err.to_string()));
        }
    };
    Ok((id_user, expected_hash_password))
}

#[instrument(name = "Verify password", skip(expected_hash, candidate))]
fn verify_password(expected_hash: &str, candidate: &str) -> bool {
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

    let is_valid = verify_password(&stored_password_hash, &req_login.password);
    if is_valid {
        return Ok(user_id);
    } else {
        return Err(CustomError::AuthenticationError(
            "Invalid credentials".to_string(),
        ));
    }
}
