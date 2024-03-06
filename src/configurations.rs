use config::Config;
use config::ConfigError;
use secrecy::ExposeSecret;
use secrecy::Secret;
use serde::Deserialize;
use serde_aux::field_attributes::deserialize_number_from_string;
use sqlx::postgres::PgConnectOptions;
use sqlx::postgres::PgSslMode;
use sqlx::ConnectOptions;
use std::convert::TryFrom;
use std::env;
use std::time::Duration;

use crate::domain::SubscriberEmail;
use crate::email_client::EmailClient;

#[derive(Deserialize, Clone)]
pub struct Settings {
    pub application: ApplicationSettings,
    pub database: DatabaseSettings,
    pub email_client: EmailClientSettings,
    pub redis_uri: Secret<String>,
}

#[derive(Deserialize, Clone)]
pub struct DatabaseSettings {
    pub username: String,
    pub password: Secret<String>,
    #[serde(deserialize_with = "deserialize_number_from_string")]
    pub port: u16,
    pub host: String,
    pub database_name: String,
    pub require_ssl: bool,
}

#[derive(Deserialize, Clone)]
pub struct ApplicationSettings {
    pub host: String,
    #[serde(deserialize_with = "deserialize_number_from_string")]
    pub port: u16,
    pub base_url: String,
    pub hmac_secret: Secret<String>,
}

#[derive(Deserialize, Clone)]
pub struct EmailClientSettings {
    pub base_url: String,
    pub sender_email: String,
    pub authorization_token: Secret<String>,
    pub timeout_milliseconds: u64,
}

impl EmailClientSettings {
    pub fn client(&self) -> EmailClient {
        let sender_email = self.sender().expect("Invalid sender email address");
        let timeout = self.timeout();
        EmailClient::new(
            self.base_url.clone(),
            sender_email,
            self.authorization_token.clone(),
            timeout,
        )
    }

    pub fn sender(&self) -> Result<SubscriberEmail, String> {
        SubscriberEmail::parse(self.sender_email.clone())
    }

    pub fn timeout(&self) -> Duration {
        Duration::from_millis(self.timeout_milliseconds)
    }
}

impl DatabaseSettings {
    pub fn without_db(&self) -> PgConnectOptions {
        let ssl_mode = if self.require_ssl {
            PgSslMode::Require
        } else {
            PgSslMode::Prefer
        };
        PgConnectOptions::new()
            .host(&self.host)
            .username(&self.username)
            .password(self.password.expose_secret())
            .port(self.port)
            .ssl_mode(ssl_mode)
    }

    pub fn with_db(&self) -> PgConnectOptions {
        let mut options = self.without_db().database(&self.database_name);
        options = options.log_statements(tracing::log::LevelFilter::Trace);
        options
    }
}

pub fn get_configuration() -> Result<Settings, ConfigError> {
    let mut settings_builder = Config::builder();

    let base_path = std::env::current_dir().expect("Fail to read the base directory");

    let configuration_path = base_path.join("configuration");

    // Read the base configuration
    settings_builder =
        settings_builder.add_source(config::File::from(configuration_path.join("base")));

    // Check if we are in production environment
    let environment: Environment = env::var("ENVIRONMENT")
        .unwrap_or("local".into())
        .try_into()
        .expect("Fail to parse environment");

    settings_builder = settings_builder.add_source(config::File::from(
        configuration_path.join(environment.as_str()),
    ));

    // Dynamically inject database secrets for deployments
    // Add in settings from environment variables (with a prefix of APP and '__' as separator)
    // E.g. `APP_APPLICATION__PORT=5001 would set `Settings.application.port`
    settings_builder =
        settings_builder.add_source(config::Environment::with_prefix("app").separator("__"));

    match settings_builder.build() {
        Ok(config) => {
            let settings: Settings = config.try_into()?;
            Ok(settings)
        }
        Err(e) => Err(e),
    }
}

enum Environment {
    Local,
    Production,
}

impl Environment {
    fn as_str(&self) -> &'static str {
        match self {
            Self::Local => "local",
            Self::Production => "production",
        }
    }
}

impl TryFrom<String> for Environment {
    type Error = String;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        match value.as_str() {
            "local" => Ok(Self::Local),
            "production" => Ok(Self::Production),
            e => Err(format!(
                "{} is not supported. Use either local or production.",
                e
            )),
        }
    }
}

impl TryFrom<Config> for Settings {
    type Error = ConfigError;

    fn try_from(config: Config) -> Result<Self, Self::Error> {
        if config.get::<ApplicationSettings>("application").is_err()
            || config.get::<DatabaseSettings>("database").is_err()
            || config.get::<EmailClientSettings>("email_client").is_err()
            || config.get::<String>("redis_uri").is_err()
        {
            return Err(ConfigError::Message(String::from("Not enough field")));
        }
        Ok(Self {
            application: config.get("application").unwrap(),
            database: config.get("database").unwrap(),
            email_client: config.get("email_client").unwrap(),
            redis_uri: Secret::new(config.get("redis_uri").unwrap()),
        })
    }
}
