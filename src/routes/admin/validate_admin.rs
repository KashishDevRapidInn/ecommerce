use super::admin::LoginAdminBody;
use crate::db::PgPool;
use crate::db_models::Admin;
use crate::schema::admins::dsl::*;
use crate::Errors::custom::CustomError;
use argon2::{self, Argon2, PasswordHash, PasswordVerifier};
use diesel::prelude::*;
use tracing::instrument;
use uuid::Uuid;

#[instrument(name = "Get stored admin credentials", skip(user_name, pool), fields(username = %user_name))]
async fn get_stored_admin_credentials(
    user_name: &str,
    pool: &PgPool,
) -> Result<(Uuid, String), CustomError> {
    let mut conn = pool.get().expect("Failed to get db connection from pool");

    let row: Result<Option<Vec<(String, Uuid)>>, diesel::result::Error> = admins
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

#[instrument(name = "Verify admin password", skip(expected_hash, candidate))]
fn verify_admin_password(expected_hash: &str, candidate: &str) -> bool {
    let argon2 = Argon2::default();
    let password_hashed = PasswordHash::new(expected_hash).expect("Failed to parse password hash");

    argon2
        .verify_password(candidate.as_bytes(), &password_hashed)
        .is_ok()
}

#[instrument(name = "Validate admin credentials", skip(req_login, pool), fields(username = %req_login.username))]
pub async fn validate_admin_credentials(
    pool: &PgPool,
    req_login: &LoginAdminBody,
) -> Result<Uuid, CustomError> {
    let (admin_id, stored_password_hash) =
        get_stored_admin_credentials(&req_login.username, pool).await?;

    let is_valid = verify_admin_password(&stored_password_hash, &req_login.password);
    if is_valid {
        return Ok(admin_id);
    } else {
        return Err(CustomError::AuthenticationError(
            "Invalid credentials".to_string(),
        ));
    }
}
