use crate::helper::{seed_products, spawn_app};
use ecommerce::db::drop_database;
use serde_json::{self, Value};
use std::time::Duration;

#[tokio::test]
async fn order_creation_get_and_list() {
    let app = spawn_app().await;

    // Step: 1= Customer login and getting jwt token
    let body = serde_json::json!({
        "username": app.test_user.username,
        "password": app.test_user.password
    });
    let login_response = app.login_customer(body).await;

    let login_response_body: Value = login_response.json().await.unwrap();
    let token = login_response_body["token"]
        .as_str()
        .expect("Token not found");
    tokio::time::sleep(Duration::from_secs(12)).await;

    // Step: 2= Adding seed data to products table
    let _ = seed_products(app.db_pool.clone()).await;

    // Step: 3= Creating New Order
    let order_body = serde_json::json!({
        "product_id": "5fcd7d83-7adf-4d4d-931a-68b9678009db",
    });
    let order_response = app.create_order(order_body, token.to_string()).await;
    assert_eq!(order_response.status().as_u16(), 200);

    let order_response_body: Value = order_response.json().await.unwrap();
    let order_id = order_response_body["order_id"]
        .as_str()
        .expect("Token not found");

    // Step: 4= Reteriving new order data and checking status is default set as pending
    let order_reterive_response = app.get_order(&order_id, token.to_string()).await;

    assert_eq!(order_reterive_response.status().as_u16(), 200);
    let order_reterive_response_text = order_reterive_response.text().await.unwrap();
    assert!(order_reterive_response_text
        .to_lowercase()
        .contains("pending"));

    // Step: 5= Reteriving all Orders by the customer
    let orders_all = app.get_all_orders(token.to_string()).await;

    let orders_all_response = orders_all.text().await.unwrap();
    assert!(orders_all_response.contains(order_id));
    assert!(orders_all_response.contains("5fcd7d83-7adf-4d4d-931a-68b9678009db"));
    drop_database(&app.database_name, app.test_db_url).await;
}
