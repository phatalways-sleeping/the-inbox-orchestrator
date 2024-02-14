use secrecy::ExposeSecret;
use sqlx::postgres::PgPoolOptions;
use std::{net::TcpListener, time::Duration};
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
    let address = format!(
        "{}:{}",
        configuration.application.host, configuration.application.port
    );
    let db_pool: sqlx::Pool<sqlx::Postgres> = PgPoolOptions::new()
        .acquire_timeout(Duration::from_secs(2))
        .connect_lazy(configuration.database.connection_string().expose_secret())
        .expect("Fail to connect to database");
    let listener = TcpListener::bind(address).expect("Fail to bind to a port");
    run(listener, db_pool)?.await
}
