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

#[derive(Deserialize)]
pub struct Settings {
    pub application: ApplicationSettings,
    pub database: DatabaseSettings,
}

#[derive(Deserialize)]
pub struct DatabaseSettings {
    pub username: String,
    pub password: Secret<String>,
    #[serde(deserialize_with = "deserialize_number_from_string")]
    pub port: u16,
    pub host: String,
    pub database_name: String,
    pub require_ssl: bool,
}

#[derive(Deserialize)]
pub struct ApplicationSettings {
    pub host: String,
    #[serde(deserialize_with = "deserialize_number_from_string")]
    pub port: u16,
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
        {
            return Err(ConfigError::Message(String::from("Not enough field")));
        }
        Ok(Self {
            application: config.get("application").unwrap(),
            database: config.get("database").unwrap(),
        })
    }
}
