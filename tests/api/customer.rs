use crate::helper::spawn_app;
use ecommerce::db::drop_database;
use serde_json::{self, Value};

#[tokio::test]
async fn customer_login_success() {
    //arrange
    let app = spawn_app().await;
    let body = serde_json::json!({
        "username": app.test_user.username,
        "password": app.test_user.password
    });

    //act
    let response = app.login_customer(body).await;
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

    // Step: 2= Updating customer body
    let update_body = serde_json::json!({
        "username": "Updated username",
        "email": "updatedemail@gmail.com"
    });
    let update_resposne = app.update_customer(update_body, token.to_string()).await;

    assert_eq!(update_resposne.status().as_u16(), 200);

    //Step: 3= Verifying using view customer
    let view_customer_response = app.view_customer(token.to_string()).await;

    assert_eq!(view_customer_response.status().as_u16(), 200);
    let body = view_customer_response.text().await.unwrap();
    println!("view customer response {:?}", body);
    assert!(body.contains("Updated username"));
    assert!(body.contains("updatedemail@gmail.com"));
    drop_database(&app.database_name);
}

#[tokio::test]
pub async fn missing_inputs_should_return_400() {
    let app = spawn_app().await;
    let test_cases = vec![
        ("name=kk%20kashyap", "missing the email"),
        ("email=kk%40gmail.com", "missing the name"),
        ("", "missing both name and email"),
    ];

    for (invalid_body, error_message) in test_cases {
        let register_response = app
            .api_client
            .post(&format!("{}/register", &app.address))
            .json(invalid_body)
            .send()
            .await
            .expect("Failed to execute request.");
        assert_eq!(
            register_response.status().as_u16(),
            400,
            "The API did not fail with 400 Bad Request when the payload was {}.",
            error_message
        )
    }
    drop_database(&app.database_name);
}
