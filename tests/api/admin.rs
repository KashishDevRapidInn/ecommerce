use crate::helper::{seed_products, spawn_app};
use ecommerce::db::drop_database;
use ecommerce::routes::order::order::OrderStatus;
use serde_json::{self, Value};
use std::time::Duration;
use uuid::Uuid;

#[tokio::test]
async fn admin_login_success() {
    let app = spawn_app().await;

    // Step: 1: Admin login and getting jwt token
    let response = app
        .api_client
        .post(&format!("{}/admin/login", &app.address))
        .json(&serde_json::json!({
            "username": app.test_user.username,
            "password": app.test_user.password
        }))
        .send()
        .await
        .expect("Failed to execute request.");

    drop_database(&app.database_name, app.test_db_url).await;

    // Step: 2= Assert
    let status_code = response.status();
    println!("Response status: {:?}", status_code);
    assert!(response.status().is_success());
    let response_body = response.json::<serde_json::Value>().await.unwrap();
    assert!(
        response_body.get("token").is_some(),
        "JWT token not found in response"
    );
}

#[tokio::test]
async fn order_creation_get_and_list() {
    let app = spawn_app().await;

    // Step: 1: Customer login and getting jwt token
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
    let order_create_body = serde_json::json!({
        "product_id": "5fcd7d83-7adf-4d4d-931a-68b9678009db",
    });
    let order_response = app.create_order(order_create_body, token.to_string()).await;

    assert_eq!(order_response.status().as_u16(), 200);
    let order_response_body: Value = order_response.json().await.unwrap();
    let order_id = order_response_body["order_id"]
        .as_str()
        .expect("Token not found");

    // Step: 4= Admin login and getting jwt token for admin
    let admin_login_body = serde_json::json!({
        "username": app.test_user.username,
        "password": app.test_user.password
    });
    let admin_login_response = app.login_admin(admin_login_body).await;

    let admin_login_response: Value = admin_login_response.json().await.unwrap();
    let admin_token = admin_login_response["token"]
        .as_str()
        .expect("Token not found");
    tokio::time::sleep(Duration::from_secs(12)).await;

    // Step: 5= Updating order status
    let update_status_body = serde_json::json!({
        "order_id": order_id,
        "status": OrderStatus::Shipped
    });
    let _update_status_response = app
        .update_order_status(update_status_body, admin_token.to_string())
        .await;

    // Step: 6= Checking order status is updated properly
    let order_reterive_response = app.get_order(order_id, token.to_string()).await;
    let order_reterive_response_text = order_reterive_response.text().await.unwrap();
    assert!(order_reterive_response_text
        .to_lowercase()
        .contains("shipped"));
    drop_database(&app.database_name, app.test_db_url).await;
}

#[tokio::test]
async fn admin_logout_check() {
    let app = spawn_app().await;

    // Step: 1= Admin login and getting jwt token
    let admin_login_body = serde_json::json!({
        "username": app.test_user.username,
        "password": app.test_user.password
    });
    let admin_login_response = app.login_admin(admin_login_body).await;

    let admin_login_response: Value = admin_login_response.json().await.unwrap();
    let admin_token = admin_login_response["token"]
        .as_str()
        .expect("Token not found");
    tokio::time::sleep(Duration::from_secs(12)).await;
    // Step: 2= Logout Admin
    let logout_response = app.logout_admin(admin_token.to_string()).await;

    assert_eq!(logout_response.status().as_u16(), 200);
    tokio::time::sleep(Duration::from_secs(12)).await;

    //Step: 3= Verifying logout
    let update_status_body = serde_json::json!({
        "order_id": Uuid::new_v4(),
        "status":  OrderStatus::Shipped
    });
    let update_status_response = app
        .update_order_status(update_status_body, admin_token.to_string())
        .await;

    assert_eq!(update_status_response.status().as_u16(), 401);
    drop_database(&app.database_name, app.test_db_url).await;
}
