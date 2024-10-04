use super::validate_admin::validate_admin_credentials;
use crate::auth_jwt::auth::create_jwt;
use crate::db::PgPool;
use crate::errors::custom::CustomError;
use crate::schema::admins::dsl as admin_dsl;
use crate::schema::orders::dsl as orders;
use crate::session_state::TypedSession;
use crate::validations::name_email::UserName;
use actix_web::{web, HttpResponse, Responder};
use argon2::{self, password_hash::SaltString, Argon2, PasswordHasher};
use diesel::prelude::*;
use rand::Rng;
use serde::Deserialize;
use serde_json;
use tracing::instrument;
use uuid::Uuid;

#[derive(Deserialize)]
pub struct CreateAdminBody {
    username: String,
    password: String,
}
impl CreateAdminBody {
    pub fn validate(self) -> Result<UserName, String> {
        let user_name = UserName::parse(self.username)?;
        Ok(user_name)
    }
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
    let validated_name = admin_data
        .validate()
        .map_err(|err| CustomError::ValidationError(err.to_string()))?;
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
                admin_dsl::username.eq(validated_name.as_ref()),
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
            let _ = session.insert_admin_id(uuid);
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
            let token = create_jwt(&id_admin.unwrap().to_string())
                .map_err(|err| CustomError::AuthenticationError(err.to_string()))?;
            let _ = session.insert_admin_id(admin_id);
            Ok(HttpResponse::Ok().json(serde_json::json!({"token": token})))
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
        .map_err(|_| CustomError::AuthenticationError("User not logged in".to_string()))?;
    session.renew();
    let mut conn = pool.get().expect("Failed to get db connection from Pool");
    let data = req_update.into_inner();
    if admin_id.is_none() {
        return Err(CustomError::AuthenticationError(
            "User not found".to_string(),
        ));
    }

    let _admin_id = admin_id.unwrap();
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
