use crate::{
    db::PgPool, errors::custom::CustomError, schema::orders::dsl as order,
    session_state::TypedSession,
};
use actix_web::{web, HttpResponse};
use diesel::prelude::*;
use tracing::instrument;
use uuid::Uuid;

#[derive(serde::Deserialize)]
pub struct CreateOrder {
    pub product_id: Uuid,
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
    let customer_id = session
        .get_user_id()
        .map_err(|_| CustomError::AuthenticationError("User not logged in".to_string()))?;
    let pool = pool.clone();
    let order_data = req_order.into_inner();
    let order_id = Uuid::new_v4();
    let order_created_at = chrono::Local::now().naive_utc();
    if customer_id.is_none() {
        return Err(CustomError::AuthenticationError(
            "User not found".to_string(),
        ));
    }

    let customer_id = customer_id.unwrap();
    let result = web::block(move || {
        let mut conn = pool.get().expect("Failed to get db connection from Pool");
        diesel::insert_into(order::orders)
            .values((
                order::id.eq(order_id),
                order::customer_id.eq(customer_id),
                order::product_id.eq(order_data.product_id),
                order::created_at.eq(order_created_at),
                order::status.eq("pending".to_string()),
            ))
            .execute(&mut conn)
            .map_err(|err| CustomError::QueryError(err.to_string()))?;
        Ok::<_, CustomError>("Order created successfully".to_string())
    })
    .await
    .map_err(|err| CustomError::QueryError(err.to_string()))?;

    match result {
        Ok(message) => {
            Ok(HttpResponse::Ok()
                .json(serde_json::json!({"message": message, "order_id": order_id})))
        }
        Err(err) => return Err(CustomError::QueryError(err.to_string())),
    }
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
    let customer_id = session
        .get_user_id()
        .map_err(|_| CustomError::AuthenticationError("User not logged in".to_string()))?;
    if customer_id.is_none() {
        return Err(CustomError::AuthenticationError(
            "User not found".to_string(),
        ));
    }

    let _customer_id = customer_id.unwrap();
    let mut conn = pool.get().expect("Failed to get db connection from Pool");
    let order: (Uuid, Uuid, String) = order::orders
        .filter(order::id.eq(order_id.into_inner()))
        .select((order::product_id, order::customer_id, order::status))
        .first(&mut conn)
        .map_err(|_| CustomError::AuthenticationError("User not logged in".to_string()))?;

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
    let customer_id = session
        .get_user_id()
        .map_err(|_| CustomError::AuthenticationError("User not logged in".to_string()))?;
    if customer_id.is_none() {
        return Err(CustomError::AuthenticationError(
            "User not found".to_string(),
        ));
    }

    let customer_id = customer_id.unwrap();

    let mut conn = pool.get().expect("Failed to get db connection from Pool");

    let order = order::orders
        .filter(order::customer_id.eq(customer_id))
        .select((order::id, order::product_id))
        .load::<(Uuid, Uuid)>(&mut conn)
        .map_err(|_| CustomError::AuthenticationError("User not logged in".to_string()))?;

    Ok(HttpResponse::Ok().json(order))
}
