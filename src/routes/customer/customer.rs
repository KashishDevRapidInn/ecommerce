use super::validate_customer::validate_credentials;
use crate::auth_jwt::auth::create_jwt;
use crate::db::PgPool;
use crate::errors::custom::{AuthError, CustomError, DbError};
use crate::schema::customers::dsl::*;
use crate::session_state::TypedSession;
use crate::validations::name_email::{UserEmail, UserName};
use actix_web::{web, HttpResponse};
use argon2::{self, password_hash::SaltString, Argon2, PasswordHasher};
use diesel::prelude::*;
use diesel_async::RunQueryDsl;
use rand::Rng;
use serde::Deserialize;
use serde_json::json;
use tracing::instrument;
use uuid::Uuid;

#[derive(Deserialize)]
pub struct CreateCustomerBody {
    username: String,
    password: String,
    email: String,
}
impl CreateCustomerBody {
    pub fn validate(self) -> Result<(UserName, UserEmail), String> {
        let user_name = UserName::parse(self.username)?;
        let user_email = UserEmail::parse(self.email)?;
        Ok((user_name, user_email))
    }
}
#[derive(Deserialize)]
pub struct UpdateCustomerBody {
    username: String,
    email: String,
}
impl UpdateCustomerBody {
    pub fn validate(self) -> Result<(UserName, UserEmail), String> {
        let user_name = UserName::parse(self.username)?;
        let user_email = UserEmail::parse(self.email)?;
        Ok((user_name, user_email))
    }
}

#[derive(Deserialize)]
pub struct LoginCustomerBody {
    pub username: String,
    pub password: String,
}

fn generate_random_salt() -> SaltString {
    let mut rng = rand::thread_rng();
    SaltString::generate(&mut rng)
}

/******************************************/
// Registering Customer Route
/******************************************/
/**
 * @route   POST /register
 * @access  Public
 */
#[instrument(name = "Register a new customer", skip(req_user, pool, session), fields(username = %req_user.username, email = %req_user.email))]
pub async fn register_customer(
    pool: web::Data<PgPool>,
    req_user: web::Json<CreateCustomerBody>,
    session: TypedSession,
) -> Result<HttpResponse, CustomError> {
    let pool = pool.clone();
    let customer_data = req_user.into_inner();
    let user_password = customer_data.password.clone();
    let (validated_name, validated_email) = customer_data
        .validate()
        .map_err(|err| CustomError::ValidationError(err.to_string()))?;
    let uuid = Uuid::new_v4();
    let mut conn = pool
        .get()
        .await
        .expect("Failed to get db connection from Pool");
    let argon2 = Argon2::default();

    let salt = generate_random_salt();
    let password_hashed = argon2
        .hash_password(user_password.as_bytes(), &salt)
        .map_err(|err| CustomError::HashingError(err.to_string()))?;

    let result = diesel::insert_into(customers)
        .values((
            id.eq(uuid),
            username.eq(validated_name.as_ref()),
            password_hash.eq(password_hashed.to_string()),
            email.eq(validated_email.as_ref()),
        ))
        .execute(&mut conn)
        .await
        .map_err(|err| CustomError::DatabaseError(DbError::QueryBuilderError(err.to_string())))?;
    if result == 0 {
        return Err(CustomError::DatabaseError(DbError::InsertionError(
            "Failed data insertion in db".to_string(),
        )));
    }
    let _ = session.insert_user_id(uuid);
    Ok(HttpResponse::Ok().body("User created successfully".to_string()))
}

/******************************************/
// Login Route
/******************************************/
/**
 * @route   POST /login
 * @access  Public
 */
#[instrument(name = "Login a customer", skip(req_login, pool, session), fields(username = %req_login.username))]

pub async fn login_customer(
    pool: web::Data<PgPool>,
    req_login: web::Json<LoginCustomerBody>,
    session: TypedSession,
) -> Result<HttpResponse, CustomError> {
    let user_id = validate_credentials(&pool, &req_login.into_inner()).await;

    match user_id {
        Ok(id_user) => {
            let token = create_jwt(&id_user.to_string()).map_err(|err| {
                CustomError::AuthenticationError(AuthError::JwtAuthenticationError(err.to_string()))
            })?;
            let _ = session.insert_user_id(id_user);
            Ok(HttpResponse::Ok().json(json!({"token": token})))
        }
        Err(err) => {
            return Err(CustomError::AuthenticationError(
                AuthError::OtherAuthenticationError(err.to_string()),
            ));
        }
    }
}

/******************************************/
// Logout Customer Route
/******************************************/
/**
 * @route   POST /protected/logout
 * @access  JWT Protected
 */
#[instrument(name = "Logout a customer", skip(session))]
pub async fn logout_customer(session: TypedSession) -> HttpResponse {
    session.log_out();
    HttpResponse::Ok().body("Logout successfull")
}

/******************************************/
// Updating Customer Profile Route
/******************************************/
/**
 * @route   POST /protected/update
 * @access  JWT Protected
 */
#[instrument(name = "Update customer", skip(req_user, pool, session), fields(username = %req_user.username, email = %req_user.email))]
pub async fn update_customer(
    pool: web::Data<PgPool>,
    req_user: web::Json<UpdateCustomerBody>,
    session: TypedSession,
) -> Result<HttpResponse, CustomError> {
    let user_id = session.get_user_id().map_err(|_| {
        CustomError::AuthenticationError(AuthError::SessionAuthenticationError(
            "User not found".to_string(),
        ))
    })?;
    let pool = pool.clone();
    let customer_data = req_user.into_inner();
    let (validated_name, validated_email) = customer_data
        .validate()
        .map_err(|err| CustomError::ValidationError(err.to_string()))?;
    if user_id.is_none() {
        return Err(CustomError::AuthenticationError(
            AuthError::SessionAuthenticationError("User not found".to_string()),
        ));
    }

    let user_id = user_id.unwrap();
    let mut conn = pool
        .get()
        .await
        .expect("Failed to get db connection from Pool");
    let result = diesel::update(customers.find(user_id))
        .set((
            username.eq(validated_name.as_ref()),
            email.eq(validated_email.as_ref()),
        ))
        .execute(&mut conn)
        .await
        .map_err(|err| CustomError::DatabaseError(DbError::QueryBuilderError(err.to_string())))?;

    if result == 0 {
        return Err(CustomError::DatabaseError(DbError::UpdationError(
            "Failed data update data in db".to_string(),
        )));
    }

    // If successful, respond with a success message
    Ok(HttpResponse::Ok().body("User updated successfully".to_string()))
}

/******************************************/
// View Customer Info Route
/******************************************/
/**
 * @route   Get /protected/view
 * @access  JWT Protected
 */
#[instrument(name = "Get customer", skip(pool, session))]
pub async fn view_customer(
    pool: web::Data<PgPool>,
    session: TypedSession,
) -> Result<HttpResponse, CustomError> {
    let user_id = session.get_user_id().map_err(|_| {
        CustomError::AuthenticationError(AuthError::SessionAuthenticationError(
            "User not found".to_string(),
        ))
    })?;
    let mut conn = pool
        .get()
        .await
        .expect("Failed to get db connection from Pool");
    if user_id.is_none() {
        return Err(CustomError::AuthenticationError(
            AuthError::SessionAuthenticationError("User not found".to_string()),
        ));
    }
    let user_id = user_id.unwrap();

    println!("User session: {}", user_id);
    let customer: (String, String) = customers
        .filter(id.eq(user_id))
        .select((username, email))
        .first(&mut conn)
        .await // if used load then I would have got Vec<(String, String)>
        .map_err(|err| CustomError::DatabaseError(DbError::QueryBuilderError(err.to_string())))?;
    Ok(HttpResponse::Ok().json(customer))
}
