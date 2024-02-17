use zero2prod::{
    configurations::get_configuration,
    startup::Application,
    telemetry::{get_subscriber, init_subscriber},
};

#[tokio::main]
async fn main() -> std::io::Result<()> {
    let subscriber = get_subscriber("zero2prod".into(), "info".into(), std::io::stdout);
    init_subscriber(subscriber);
    let configuration = get_configuration().expect("Cannot read configuration");

    let application = Application::build(configuration).await?;

    application.run_until_stopped().await?; // Run the server 

    Ok(())
}
