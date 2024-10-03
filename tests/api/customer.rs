use crate::helper::spawn_app;
use ecommerce::db::drop_database;
use serde_json::{self, Value};

#[tokio::test]
async fn customer_login_success() {
    //arrange
    let app = spawn_app().await;

    //act
    let response = app
        .api_client
        .post(&format!("{}/login", &app.address))
        .json(&serde_json::json!({
            "username": app.test_user.username,
            "password": app.test_user.password
        }))
        .send()
        .await
        .expect("Failed to execute request.");

    drop_database(&app.database_name);

    //assert
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
async fn update_customer_and_view_customer_route_testing() {
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

    let update_resposne = app
        .api_client
        .post(&format!("{}/protected/update", &app.address))
        .bearer_auth(token)
        .json(&serde_json::json!({
            "username": "Updated username",
            "email": "updatedemail@gmail.com"
        }))
        .send()
        .await
        .expect("Failed to execute request.");

    assert_eq!(update_resposne.status().as_u16(), 200);

    let view_customer_response = app
        .api_client
        .get(&format!("{}/protected/view", &app.address))
        .bearer_auth(token)
        .send()
        .await
        .expect("Failed to execute request");

    assert_eq!(view_customer_response.status().as_u16(), 200);
    let body = view_customer_response.text().await.unwrap();
    println!("view customer response {:?}", body);
    assert!(body.contains("Updated username"));
    assert!(body.contains("updatedemail@gmail.com"));
    drop_database(&app.database_name);
}
