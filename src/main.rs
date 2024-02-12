use secrecy::ExposeSecret;
use sqlx::PgPool;
use std::net::TcpListener;
use zero2prod::{
    configurations::get_configuration,
    run,
    telemetry::{get_subscriber, init_subscriber},
};

#[tokio::main]
async fn main() -> std::io::Result<()> {
    let subscriber = get_subscriber("zero2prod".into(), "info".into(), std::io::stdout);
    init_subscriber(subscriber);
    let configuration = get_configuration().expect("Cannot read configuration");
    let address = format!("127.0.0.1:{}", configuration.application_port);
    let db_pool = PgPool::connect(configuration.database.connection_string().expose_secret())
        .await
        .expect("Fail to connect to database");
    let listener = TcpListener::bind(address).expect("Fail to bind to a port");
    run(listener, db_pool)?.await
}
