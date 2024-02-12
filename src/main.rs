use std::net::TcpListener;

use zero2prod::{configurations::get_configuration, run};

#[tokio::main]
async fn main() -> std::io::Result<()> {
    let configuration = get_configuration().expect("Cannot read configuration");
    let address = format!("127.0.0.1:{}", configuration.application_port);
    let listener = TcpListener::bind(address).expect("Fail to bind to a port");
    run(listener)?.await
}
