use reqwest::Client;

use crate::helpers::spawn_app;

#[tokio::test]
async fn health_check_works() {
    let test_app = spawn_app().await;
    let client = Client::new();
    let response = client
        .get(&format!("{}/health_check", test_app.address))
        .send()
        .await
        .expect("Failed to send");
    assert!(response.status().is_success());
    assert_eq!(Some(0), response.content_length());
}
