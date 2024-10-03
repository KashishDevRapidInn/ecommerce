use crate::helper::spawn_app;
// use ecommerce::db::drop_database;
use reqwest::Client;

#[tokio::test]
async fn health_check_works() {
    let app = spawn_app().await;
    let client = Client::new();
    let response = client
        .get(&format!("{}/health_check", &app.address))
        .send()
        .await
        .expect("Failed to execute request.");
    let status_code = response.status();
    println!("Response status: {:?}", status_code);
    assert!(response.status().is_success());
    assert_eq!(Some(0), response.content_length());
    // drop_database(&app.database_name);
}
