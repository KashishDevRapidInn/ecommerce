use super::validate_customer::validate_credentials;
use crate::auth_jwt::auth::create_jwt;
use crate::db::PgPool;
use crate::schema::customers::dsl::*;
use crate::session_state::TypedSession;
use crate::validations::customer::{CustomerEmail, CustomerName};
use crate::Errors::custom::CustomError;
use actix_web::{web, HttpResponse, Responder};
use argon2::{self, password_hash::SaltString, Argon2, PasswordHasher};
use diesel::prelude::*;
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
    pub fn validate(self) -> Result<(CustomerName, CustomerEmail), String> {
        let user_name = CustomerName::parse(self.username)?;
        let user_email = CustomerEmail::parse(self.email)?;
        Ok((user_name, user_email))
    }
}
#[derive(Deserialize)]
pub struct UpdateCustomerBody {
    username: String,
    email: String,
}
impl UpdateCustomerBody {
    pub fn validate(self) -> Result<(CustomerName, CustomerEmail), String> {
        let user_name = CustomerName::parse(self.username)?;
        let user_email = CustomerEmail::parse(self.email)?;
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
    let result = web::block(move || {
        let mut conn = pool.get().expect("Failed to get db connection from Pool");
        let argon2 = Argon2::default();

        let salt = generate_random_salt();
        let password_hashed = argon2
            .hash_password(user_password.as_bytes(), &salt)
            .map_err(|err| CustomError::HashingError(err.to_string()))?;

        // let user_name = customer_data.username.clone();
        // let user_email = customer_data.email.clone();
        diesel::insert_into(customers)
            .values((
                id.eq(uuid),
                username.eq(validated_name.as_ref()),
                password_hash.eq(password_hashed.to_string()),
                email.eq(validated_email.as_ref()),
            ))
            .execute(&mut conn)
            .map_err(|err| CustomError::QueryError(err.to_string()))?;

        Ok::<_, CustomError>("User created successfully".to_string())
    })
    .await
    .map_err(|err| CustomError::BlockingError(err.to_string()))?;

    match result {
        Ok(message) => {
            session.insert_user_id(uuid);
            Ok(HttpResponse::Ok().body(message))
        }
        Err(err) => Err(err),
    }
}

#[instrument(name = "Login a customer", skip(req_login, pool, session), fields(username = %req_login.username))]

pub async fn login_customer(
    pool: web::Data<PgPool>,
    req_login: web::Json<LoginCustomerBody>,
    session: TypedSession,
) -> Result<HttpResponse, CustomError> {
    let user_id = validate_credentials(&pool, &req_login.into_inner()).await;

    match user_id {
        Ok(id_user) => {
            let token = create_jwt(&id_user.to_string())
                .map_err(|err| CustomError::AuthenticationError(err.to_string()))?;
            session.insert_user_id(id_user);
            Ok(HttpResponse::Ok().json(json!({"token": token})))
        }
        Err(err) => {
            return Err(CustomError::AuthenticationError(err.to_string()))?;
        }
    }
}
#[instrument(name = "Logout a customer", skip(session))]
pub async fn logout_customer(session: TypedSession) -> impl Responder {
    session.log_out();
    HttpResponse::Ok().body("Login successfull")
}

#[instrument(name = "Update customer", skip(req_user, pool, session), fields(username = %req_user.username, email = %req_user.email))]
pub async fn update_customer(
    pool: web::Data<PgPool>,
    req_user: web::Json<UpdateCustomerBody>,
    session: TypedSession,
) -> Result<HttpResponse, CustomError> {
    let user_id = session
        .get_user_id()
        .map_err(|_| CustomError::AuthenticationError("User not logged in".to_string()))?;
    let pool = pool.clone();
    let customer_data = req_user.into_inner();
    let (validated_name, validated_email) = customer_data
        .validate()
        .map_err(|err| CustomError::ValidationError(err.to_string()))?;
    if user_id.is_none() {
        return Err(CustomError::AuthenticationError(
            "User not found".to_string(),
        ));
    }

    let user_id = user_id.unwrap();
    let result = web::block(move || {
        let mut conn = pool.get().expect("Failed to get db connection from Pool");
        diesel::update(customers.find(user_id))
            .set((
                username.eq(validated_name.as_ref()),
                email.eq(validated_email.as_ref()),
            ))
            .execute(&mut conn)
            .map_err(|err| CustomError::QueryError(err.to_string()))?;

        Ok::<_, CustomError>("User Updated successfully".to_string())
    })
    .await
    .map_err(|err| CustomError::BlockingError(err.to_string()))?;

    match result {
        Ok(message) => {
            session.insert_user_id(user_id);
            Ok(HttpResponse::Ok().body(message))
        }
        Err(err) => Err(err),
    }
}

#[instrument(name = "Get customer", skip(pool, session))]
pub async fn view_customer(
    pool: web::Data<PgPool>,
    session: TypedSession,
) -> Result<HttpResponse, CustomError> {
    let user_id = session
        .get_user_id()
        .map_err(|_| CustomError::AuthenticationError("User not logged in".to_string()))?;
    let mut conn = pool.get().expect("Failed to get db connection from Pool");
    if user_id.is_none() {
        return Err(CustomError::AuthenticationError(
            "User not found".to_string(),
        ));
    }

    let user_id = user_id.unwrap();
    let customer: (String, String) = customers
        .filter(id.eq(user_id))
        .select((username, email))
        .first(&mut conn) // if used load then I would have got Vec<(String, String)>
        .map_err(|err| CustomError::QueryError(err.to_string()))?;
    Ok(HttpResponse::Ok().json(customer))
}
