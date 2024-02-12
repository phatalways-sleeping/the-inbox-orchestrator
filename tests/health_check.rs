use std::net::TcpListener;

use reqwest::Client;
use sqlx::{Connection, Executor, PgConnection, PgPool};
use uuid::Uuid;
use zero2prod::configurations::{get_configuration, DatabaseSettings};

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

#[tokio::test]
async fn subcribe_returns_200_for_valid_form_data() {
    let test_app = spawn_app().await;
    let client = Client::new();
    let body = "username=le%20guin&email=ursula_le_guin%40gmail.com";
    let response = client
        .post(&format!("{}/subscriptions", test_app.address))
        .header("Content-Type", "application/x-www-form-urlencoded")
        .body(body)
        .send()
        .await
        .expect("Fail to send");

    assert_eq!(response.status().as_u16(), 200);
    let saved = sqlx::query!("SELECT email, username from subscriptions")
        .fetch_one(&test_app.db_pool)
        .await
        .expect("Failed to fetch saved subscriptions");
    assert_eq!(saved.username, "le guin");
    assert_eq!(saved.email, "ursula_le_guin@gmail.com");
}

#[tokio::test]
async fn subscribe_returns_400_for_bad_requests() {
    let test_app = spawn_app().await;
    let client = Client::new();
    let testcases = vec![
        ("username=le%20guin", "missing the email"),
        ("email=ursula_le_guin%40gmail.com", "missing the username"),
        ("", "missing both username and email"),
    ];
    for (body, err_msg) in testcases {
        let response = client
            .post(&format!("{}/subscriptions", test_app.address))
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

pub struct TestApp {
    pub address: String,
    pub db_pool: PgPool,
}

async fn spawn_app() -> TestApp {
    let listener = TcpListener::bind("127.0.0.1:0").expect("Fail to bind to a random port");
    let port = listener.local_addr().unwrap().port();
    let mut configuration = get_configuration().expect("Fail to read configuration");
    configuration.database.database_name = Uuid::new_v4().to_string();
    let conection_pool = configure_database(&configuration.database).await;
    let server =
        zero2prod::run(listener, conection_pool.clone()).expect("Fail to bind to the address");
    let _ = tokio::spawn(server);

    TestApp {
        address: format!("http://127.0.0.1:{}", port),
        db_pool: conection_pool,
    }
}

pub async fn configure_database(config: &DatabaseSettings) -> PgPool {
    let mut connection = PgConnection::connect(&config.raw_connection_string())
        .await
        .expect("Fail to connect to Postgres server");
    connection
        .execute(format!(r#"CREATE DATABASE "{}";"#, config.database_name).as_str())
        .await
        .expect("Fail to create new table on the server");
    // Run the migration on the database
    let connection_pool = PgPool::connect(&config.connection_string())
        .await
        .expect("Fail to connect to the newly created database");
    sqlx::migrate!("./migrations")
        .run(&connection_pool)
        .await
        .expect("Fail to run migration");
    connection_pool
}
