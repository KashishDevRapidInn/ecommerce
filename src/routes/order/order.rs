use crate::{
    db::PgPool,
    errors::custom::{AuthError, CustomError, DbError},
    schema::orders::dsl as order,
    session_state::TypedSession,
};
use actix_web::{web, HttpResponse};
use diesel::prelude::*;
use diesel_async::{AsyncConnection, RunQueryDsl};
use diesel_derive_enum;
use tracing::instrument;
use uuid::Uuid;

#[derive(serde::Deserialize)]
pub struct CreateOrder {
    pub product_id: Uuid,
}
#[derive(Debug, diesel_derive_enum::DbEnum, serde::Serialize, serde::Deserialize)]
#[ExistingTypePath = "crate::schema::sql_types::OrderStatus"]
pub enum OrderStatus {
    Pending,
    Shipped,
    Delivered,
}
/******************************************/
// New Order Creation route
/******************************************/
/**
 * @route   POST /protected/orders/new
 * @access  JWT Protected
 */
#[instrument(name = "Create new Order", skip(req_order, pool, session))]
pub async fn create_order(
    pool: web::Data<PgPool>,
    req_order: web::Json<CreateOrder>,
    session: TypedSession,
) -> Result<HttpResponse, CustomError> {
    let customer_id = session.get_user_id().map_err(|_| {
        CustomError::AuthenticationError(AuthError::SessionAuthenticationError(
            "User not found".to_string(),
        ))
    })?;
    let pool = pool.clone();
    let order_data = req_order.into_inner();
    let order_id = Uuid::new_v4();
    let order_created_at = chrono::Local::now().naive_utc();
    if customer_id.is_none() {
        return Err(CustomError::AuthenticationError(
            AuthError::SessionAuthenticationError("User not found".to_string()),
        ));
    }

    let customer_id = customer_id.unwrap();
    let mut conn = pool
        .get()
        .await
        .expect("Failed to get db connection from Pool");
    let result = diesel::insert_into(order::orders)
        .values((
            order::id.eq(order_id),
            order::customer_id.eq(customer_id),
            order::product_id.eq(order_data.product_id),
            order::created_at.eq(order_created_at),
            order::status.eq(OrderStatus::Pending),
        ))
        .execute(&mut conn)
        .await
        .map_err(|err| CustomError::DatabaseError(DbError::QueryBuilderError(err.to_string())))?;
    if result == 0 {
        return Err(CustomError::DatabaseError(DbError::UpdationError(
            "Failed data update data in db".to_string(),
        )));
    }

    Ok(HttpResponse::Ok().json(serde_json::json!({"message": "Order created successfully".to_string(), "order_id": order_id})))
}

/******************************************/
// Reteriving Order using id
/******************************************/
/**
 * @route   Get /protected/orders/{id}/view
 * @access  JWT Protected
 */
#[instrument(name = "Get Order", skip(order_id, pool, session))]
pub async fn get_order(
    pool: web::Data<PgPool>,
    order_id: web::Path<Uuid>,
    session: TypedSession,
) -> Result<HttpResponse, CustomError> {
    let customer_id = session.get_user_id().map_err(|_| {
        CustomError::AuthenticationError(AuthError::SessionAuthenticationError(
            "User not found".to_string(),
        ))
    })?;
    if customer_id.is_none() {
        return Err(CustomError::AuthenticationError(
            AuthError::SessionAuthenticationError("User not found".to_string()),
        ));
    }

    let _customer_id = customer_id.unwrap();
    let mut conn = pool
        .get()
        .await
        .expect("Failed to get db connection from Pool");
    let order: (Uuid, Uuid, OrderStatus) = order::orders
        .filter(order::id.eq(order_id.into_inner()))
        .select((order::product_id, order::customer_id, order::status))
        .first(&mut conn)
        .await
        .map_err(|err| CustomError::DatabaseError(DbError::QueryBuilderError(err.to_string())))?;

    Ok(HttpResponse::Ok().json(order))
}

/******************************************/
// Reteriving All Orders of a customer
/******************************************/
/**
 * @route   Get /protected/orders/list/all
 * @access  JWT Protected
 */
#[instrument(name = "Get All Orders by customer", skip(pool, session))]
pub async fn list_orders(
    pool: web::Data<PgPool>,
    session: TypedSession,
) -> Result<HttpResponse, CustomError> {
    let customer_id = session.get_user_id().map_err(|_| {
        CustomError::AuthenticationError(AuthError::SessionAuthenticationError(
            "User not found".to_string(),
        ))
    })?;
    if customer_id.is_none() {
        return Err(CustomError::AuthenticationError(
            AuthError::SessionAuthenticationError("User not found".to_string()),
        ));
    }

    let customer_id = customer_id.unwrap();

    let mut conn = pool
        .get()
        .await
        .expect("Failed to get db connection from Pool");

    let order = order::orders
        .filter(order::customer_id.eq(customer_id))
        .select((order::id, order::product_id))
        .load::<(Uuid, Uuid)>(&mut conn)
        .await
        .map_err(|err| CustomError::DatabaseError(DbError::QueryBuilderError(err.to_string())))?;

    Ok(HttpResponse::Ok().json(order))
}
