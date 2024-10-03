use crate::helper::spawn_app;
use ecommerce::db::drop_database;
use serde_json::{self, Value};

#[tokio::test]
async fn admin_login_success() {
    //arrange
    let app = spawn_app().await;
    // let client = Client::new();

    //act
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
