use super::validate_admin::validate_admin_credentials;
use crate::db::PgPool;
use crate::routes::customer::customer_error::CustomerError;
use crate::schema::admins::dsl as admin_dsl;
use crate::session_state::TypedSession;
use actix_web::{web, HttpResponse, Responder};
use argon2::{
    self, password_hash::SaltString, Argon2, PasswordHash, PasswordHasher, PasswordVerifier,
};
use diesel::prelude::*;
use rand::Rng;
use serde::Deserialize;
use tracing::{error, info, instrument};
use uuid::Uuid;
#[derive(Deserialize)]
pub struct CreateAdminBody {
    username: String,
    password: String,
}

#[derive(Deserialize)]
pub struct LoginAdminBody {
    pub username: String,
    pub password: String,
}

fn generate_random_salt() -> SaltString {
    let mut rng = rand::thread_rng();
    SaltString::generate(&mut rng)
}

#[instrument(name = "Register Admin", skip(req_admin, pool, session))]
pub async fn register_admin(
    pool: web::Data<PgPool>,
    req_admin: web::Json<CreateAdminBody>,
    session: TypedSession,
) -> Result<HttpResponse, CustomerError> {
    let pool = pool.clone();
    let admin_data = req_admin.into_inner();
    let admin_password = admin_data.password.clone();
    let admin_username = admin_data.username.clone();
    let uuid = Uuid::new_v4();
    let result = web::block(move || {
        let mut conn = pool.get().expect("Failed to get db connection from Pool");
        let argon2 = Argon2::default();

        let salt = generate_random_salt();
        let password_hashed = argon2
            .hash_password(admin_password.as_bytes(), &salt)
            .map_err(|err| CustomerError::HashingError(err.to_string()))?;

        diesel::insert_into(admin_dsl::admins)
            .values((
                admin_dsl::id.eq(uuid),
                admin_dsl::username.eq(admin_username),
                admin_dsl::password_hash.eq(password_hashed.to_string()),
            ))
            .execute(&mut conn)
            .map_err(|err| CustomerError::QueryError(err.to_string()))?;

        Ok::<_, CustomerError>("Admin created successfully".to_string())
    })
    .await
    .map_err(|err| CustomerError::BlockingError(err.to_string()))?;

    match result {
        Ok(message) => {
            session.insert_admin_id(uuid);
            Ok(HttpResponse::Ok().body(message))
        }
        Err(err) => Err(err),
    }
}

#[instrument(name = "Login admin", skip(req_login, pool, session))]

pub async fn login_admin(
    pool: web::Data<PgPool>,
    req_login: web::Json<LoginAdminBody>,
    session: TypedSession,
) -> Result<HttpResponse, CustomerError> {
    let id_admin = validate_admin_credentials(&pool, &req_login.into_inner()).await;

    match id_admin {
        Ok(admin_id) => {
            session.insert_admin_id(admin_id);
            Ok(HttpResponse::Ok().body("Admin Login successful"))
        }
        Err(err) => {
            return Err(CustomerError::AuthenticationError(err.to_string()))?;
        }
    }
}
#[instrument(name = "Logout admin", skip(session))]
pub async fn logout_admin(session: TypedSession) -> impl Responder {
    session.admin_log_out();
    HttpResponse::Ok().body("Login successfull")
}
