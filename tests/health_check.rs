use std::net::TcpListener;

use reqwest::Client;
use sqlx::{Connection, PgConnection};
use zero2prod::configurations::get_configuration;

#[tokio::test]
async fn health_check_works() {
    let address = spawn_app();
    let client = Client::new();
    let response = client
        .get(&format!("{}/health_check", address))
        .send()
        .await
        .expect("Failed to send");
    assert!(response.status().is_success());
    assert_eq!(Some(0), response.content_length());
}

#[tokio::test]
async fn subcribe_returns_200_for_valid_form_data() {
    let address = spawn_app();
    let configuration = get_configuration().expect("Fail to read configuration");
    let connection_string = configuration.database.connection_string();
    let mut connection = PgConnection::connect(&connection_string)
        .await
        .expect("Fail to connect to Postgres");
    let saved = sqlx::query!("SELECT email, username from subscriptions")
        .fetch_one(&mut connection)
        .await
        .expect("Failed to fetch saved subscriptions");
    let client = Client::new();
    let body = "username=le%20guin&email=ursula_le_guin%40gmail.com";
    let response = client
        .post(&format!("{}/subscriptions", address))
        .header("Content-Type", "application/x-www-form-urlencoded")
        .body(body)
        .send()
        .await
        .expect("Fail to send");

    assert_eq!(response.status().as_u16(), 200);
}

#[tokio::test]
async fn subscribe_returns_400_for_bad_requests() {
    let address = spawn_app();
    let client = Client::new();
    let testcases = vec![
        ("username=le%20guin", "missing the email"),
        ("email=ursula_le_guin%40gmail.com", "missing the username"),
        ("", "missing both username and email"),
    ];
    for (body, err_msg) in testcases {
        let response = client
            .post(&format!("{}/subscriptions", &address))
            .header("Content-Type", "application/x-www-form-urlencoded")
            .body(body)
            .send()
            .await
            .expect("Fail to send");

        assert_eq!(
            response.status().as_u16(),
            400,
            "The API did not fail with 400 Bad Request when the payload was {}",
            err_msg
        );
    }
}

fn spawn_app() -> String {
    let listener = TcpListener::bind("127.0.0.1:0").expect("Fail to bind to a random port");
    let port = listener.local_addr().unwrap().port();
    let server = zero2prod::run(listener).expect("Fail to bind to the address");
    let _ = tokio::spawn(server);
    format!("http://127.0.0.1:{}", port)
}
