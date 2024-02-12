use std::net::TcpListener;

use sqlx::PgPool;
use zero2prod::{configurations::get_configuration, run};

#[tokio::main]
async fn main() -> std::io::Result<()> {
    let configuration = get_configuration().expect("Cannot read configuration");
    let address = format!("127.0.0.1:{}", configuration.application_port);
    let connection_string = configuration.database.connection_string();
    let db_pool = PgPool::connect(&connection_string)
        .await
        .expect("Fail to connect to database");
    let listener = TcpListener::bind(address).expect("Fail to bind to a port");
    run(listener, db_pool)?.await
}
