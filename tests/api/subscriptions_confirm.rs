use wiremock::{
    matchers::{method, path},
    Mock, ResponseTemplate,
};

use crate::helpers::spawn_app;

#[tokio::test]
pub async fn confirmation_without_token_is_rejected() {
    let app = spawn_app().await;

    let response = reqwest::get(&format!("{}/subscriptions/confirm", app.address))
        .await
        .unwrap();

    assert_eq!(response.status().as_u16(), 400)
}

#[tokio::test]
pub async fn the_link_received_in_subscribe_returns_200_if_called() {
    let app = spawn_app().await;
    let body = "username=le%20guin&email=ursula_le_guin%40gmail.com";

    // Mock the expected results
    Mock::given(path("/email"))
        .and(method("POST"))
        .respond_with(ResponseTemplate::new(200))
        .expect(1)
        .mount(&app.email_server)
        .await;

    // Send the request
    app.post_subscriptions(body.into()).await;

    // Intercept the server, get the first request
    let email_request = &app.email_server.received_requests().await.unwrap()[0];

    // Parse
    let confirmation_links = app.get_confirmation_link(email_request);

    assert_eq!(confirmation_links.html, confirmation_links.plain_text);

    // Send GET request
    let response = reqwest::get(confirmation_links.html).await.unwrap();

    assert_eq!(response.status().as_u16(), 200);
}

#[tokio::test]
pub async fn clicking_on_the_confirmation_link_confirms_a_subscriber() {
    let app = spawn_app().await;
    let body = "username=le%20guin&email=ursula_le_guin%40gmail.com";

    // Mock the expected results
    Mock::given(path("/email"))
        .and(method("POST"))
        .respond_with(ResponseTemplate::new(200))
        .expect(1)
        .mount(&app.email_server)
        .await;

    // Send the request
    app.post_subscriptions(body.into()).await;

    // Intercept the server, get the first request
    let email_request = &app.email_server.received_requests().await.unwrap()[0];

    // Parse
    let confirmation_links = app.get_confirmation_link(email_request);

    assert_eq!(confirmation_links.html, confirmation_links.plain_text);

    // Send GET request
    reqwest::get(confirmation_links.html)
        .await
        .unwrap()
        .error_for_status()
        .unwrap();

    // Check the status in the database
    let saved = sqlx::query!("SELECT username, email, status FROM subscriptions",)
        .fetch_one(&app.db_pool)
        .await
        .expect("Failed to fetch saved subscriptions");

    assert_eq!(saved.username, "le guin");
    assert_eq!(saved.email, "ursula_le_guin@gmail.com");
    assert_eq!(saved.status, "confirmed");
}
