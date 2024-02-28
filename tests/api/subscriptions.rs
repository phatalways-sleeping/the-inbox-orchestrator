use wiremock::{
    matchers::{method, path},
    Mock, ResponseTemplate,
};

use crate::helpers::spawn_app;

#[tokio::test]
async fn subcribe_returns_200_for_valid_form_data() {
    let app = spawn_app().await;
    let body = "username=le%20guin&email=ursula_le_guin%40gmail.com";

    // Mock the expected results
    Mock::given(path("/email"))
        .and(method("POST"))
        .respond_with(ResponseTemplate::new(200))
        .expect(1)
        .mount(&app.email_server)
        .await;

    let response = app.post_subscriptions(body.into()).await;

    assert_eq!(response.status().as_u16(), 200);
}

#[tokio::test]
async fn subcribe_persist_new_user() {
    let app = spawn_app().await;
    let body = "username=le%20guin&email=ursula_le_guin%40gmail.com";

    // Mock the expected results
    Mock::given(path("/email"))
        .and(method("POST"))
        .respond_with(ResponseTemplate::new(200))
        .expect(1)
        .mount(&app.email_server)
        .await;

    let response = app.post_subscriptions(body.into()).await;

    assert_eq!(response.status().as_u16(), 200);
    let saved = sqlx::query!("SELECT email, username, status from subscriptions")
        .fetch_one(&app.db_pool)
        .await
        .expect("Failed to fetch saved subscriptions");
    assert_eq!(saved.username, "le guin");
    assert_eq!(saved.email, "ursula_le_guin@gmail.com");
    assert_eq!(saved.status, "pending_confirmation");
}

#[tokio::test]
async fn subscribe_returns_400_when_fields_are_present_but_invalid() {
    let test_app = spawn_app().await;
    let testcases = vec![
        ("username=le%20guin&email=", "empty email"),
        (
            "username=&email=ursula_le_guin%40gmail.com",
            "empty username",
        ),
        ("username=&email=", "both email and username are empty"),
    ];
    for (body, err_msg) in testcases {
        let response = test_app.post_subscriptions(body.into()).await;

        assert_eq!(
            response.status().as_u16(),
            400,
            "The API did not return with 400 Bad Request when the payload was {}",
            err_msg
        );
    }
}

#[tokio::test]
async fn subscribe_returns_400_for_bad_requests() {
    let test_app = spawn_app().await;
    let testcases = vec![
        ("username=le%20guin", "missing the email"),
        ("email=ursula_le_guin%40gmail.com", "missing the username"),
        ("", "missing both username and email"),
    ];
    for (body, err_msg) in testcases {
        let response = test_app.post_subscriptions(body.into()).await;

        assert_eq!(
            response.status().as_u16(),
            400,
            "The API did not fail with 400 Bad Request when the payload was {}",
            err_msg
        );
    }
}

#[tokio::test]
async fn subscribe_sends_a_confirmation_email_for_valid_data() {
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
}

#[tokio::test]
async fn subscribe_sends_a_confirmation_email_with_a_link() {
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
}

#[tokio::test]
async fn subscribe_fails_if_there_is_a_fatal_error() {
    let app = spawn_app().await;
    let body = "name=le%20guin&email=ursula_le_guin%40gmail.com";
    // Sabotage the database
    sqlx::query!("ALTER TABLE subscription_tokens DROP COLUMN subscription_token;",)
        .execute(&app.db_pool)
        .await
        .unwrap();

    let response = app.post_subscriptions(body.into()).await;

    assert_eq!(response.status().as_u16(), 500);
}
