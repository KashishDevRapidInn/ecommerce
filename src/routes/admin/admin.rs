use super::validate_admin::validate_admin_credentials;
use crate::db::PgPool;
use crate::schema::admins::dsl as admin_dsl;
use crate::schema::orders::dsl as orders;
use crate::session_state::TypedSession;
use crate::Errors::custom::CustomError;
use actix_web::{web, HttpResponse, Responder};
use argon2::{
    self, password_hash::SaltString, Argon2, PasswordHash, PasswordHasher, PasswordVerifier,
};
use diesel::prelude::*;
use rand::Rng;
use serde::Deserialize;
use tracing::instrument;
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

#[derive(Deserialize)]
pub struct UpdateStatusBody {
    pub order_id: Uuid,
    pub status: String,
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
) -> Result<HttpResponse, CustomError> {
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
            .map_err(|err| CustomError::HashingError(err.to_string()))?;

        diesel::insert_into(admin_dsl::admins)
            .values((
                admin_dsl::id.eq(uuid),
                admin_dsl::username.eq(admin_username),
                admin_dsl::password_hash.eq(password_hashed.to_string()),
            ))
            .execute(&mut conn)
            .map_err(|err| CustomError::QueryError(err.to_string()))?;

        Ok::<_, CustomError>("Admin created successfully".to_string())
    })
    .await
    .map_err(|err| CustomError::BlockingError(err.to_string()))?;

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
) -> Result<HttpResponse, CustomError> {
    let id_admin = validate_admin_credentials(&pool, &req_login.into_inner()).await;

    match id_admin {
        Ok(admin_id) => {
            session.insert_admin_id(admin_id);
            Ok(HttpResponse::Ok().body("Admin Login successful"))
        }
        Err(err) => {
            return Err(CustomError::AuthenticationError(err.to_string()))?;
        }
    }
}

#[instrument(name = "Logout admin", skip(session))]
pub async fn logout_admin(session: TypedSession) -> impl Responder {
    session.admin_log_out();
    HttpResponse::Ok().body("Login successfull")
}

#[instrument(name = "Update order status admin", skip(req_update, pool, session))]
pub async fn update_status(
    pool: web::Data<PgPool>,
    req_update: web::Json<UpdateStatusBody>,
    session: TypedSession,
) -> Result<HttpResponse, CustomError> {
    let admin_id = session
        .get_admin_id()
        .map_err(|err| CustomError::AuthenticationError("User not logged in".to_string()))?;
    session.renew();
    let mut conn = pool.get().expect("Failed to get db connection from Pool");
    let data = req_update.into_inner();
    if admin_id.is_none() {
        return Err(CustomError::AuthenticationError(
            "User not found".to_string(),
        ));
    }

    let admin_id = admin_id.unwrap();
    let result: Result<String, CustomError> = web::block(move || {
        diesel::update(orders::orders.filter(orders::id.eq(data.order_id)))
            .set(orders::status.eq(data.status.to_string()))
            .execute(&mut conn)
            .map_err(|err| CustomError::QueryError(err.to_string()))?;
        Ok::<_, CustomError>("Order Status Updated successfully".to_string())
    })
    .await
    .map_err(|err| CustomError::BlockingError(err.to_string()))?;

    match result {
        Ok(message) => Ok(HttpResponse::Ok().body(message)),
        Err(err) => Err(err),
    }
}
