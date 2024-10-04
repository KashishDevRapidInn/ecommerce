use crate::helper::{seed_products, spawn_app};
use ecommerce::db::drop_database;
use serde_json::{self, Value};

#[tokio::test]
async fn order_creation_get_and_list() {
    //arrange
    let app = spawn_app().await;
    let login_response = app
        .api_client
        .post(&format!("{}/login", &app.address))
        .json(&serde_json::json!({
            "username": app.test_user.username,
            "password": app.test_user.password
        }))
        .send()
        .await
        .expect("Failed to execute request.");

    let login_response_body: Value = login_response.json().await.unwrap();
    let token = login_response_body["token"]
        .as_str()
        .expect("Token not found");

    let _ = seed_products(app.db_pool.clone());
    let order_response = app
        .api_client
        .post(&format!("{}/protected/orders/new", &app.address))
        .bearer_auth(token)
        .json(&serde_json::json!({
            "product_id": "5fcd7d83-7adf-4d4d-931a-68b9678009db",
        }))
        .send()
        .await
        .expect("Failed to execute request.");
    assert_eq!(order_response.status().as_u16(), 200);

    let order_response_body: Value = order_response.json().await.unwrap();
    let order_id = order_response_body["order_id"]
        .as_str()
        .expect("Token not found");

    let order_reterive_response = app
        .api_client
        .get(&format!(
            "{}/protected/orders/{}/view",
            &app.address, &order_id
        ))
        .bearer_auth(token)
        .send()
        .await
        .expect("Failed to execute request.");
    assert_eq!(order_reterive_response.status().as_u16(), 200);
    let order_reterive_response_text = order_reterive_response.text().await.unwrap();
    assert!(order_reterive_response_text.contains("pending"));

    let orders_all = app
        .api_client
        .get(&format!("{}/protected/orders/list/all", &app.address))
        .bearer_auth(token)
        .send()
        .await
        .expect("Failed to execute request.");

    let orders_all_response = orders_all.text().await.unwrap();
    assert!(orders_all_response.contains(order_id));
    assert!(orders_all_response.contains("5fcd7d83-7adf-4d4d-931a-68b9678009db"));
    drop_database(&app.database_name);
}
