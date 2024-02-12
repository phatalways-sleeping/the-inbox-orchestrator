use config::Config;
use config::ConfigError;
use secrecy::ExposeSecret;
use secrecy::Secret;
use serde::Deserialize;
use std::convert::TryFrom;

#[derive(Deserialize)]
pub struct Settings {
    pub application_port: u16,
    pub database: DatabaseSettings,
}

#[derive(Deserialize)]
pub struct DatabaseSettings {
    pub username: String,
    pub password: Secret<String>,
    pub port: u16,
    pub host: String,
    pub database_name: String,
}

impl DatabaseSettings {
    pub fn connection_string(&self) -> Secret<String> {
        Secret::new(format!(
            "postgres://{}:{}@{}:{}/{}",
            self.username,
            self.password.expose_secret(),
            self.host,
            self.port,
            self.database_name
        ))
    }

    pub fn raw_connection_string(&self) -> Secret<String> {
        Secret::new(format!(
            "postgres://{}:{}@{}:{}",
            self.username,
            self.password.expose_secret(),
            self.host,
            self.port
        ))
    }
}

pub fn get_configuration() -> Result<Settings, ConfigError> {
    let settings_builder = Config::builder();

    match settings_builder
        .add_source(config::File::with_name("configuration"))
        .build()
    {
        Ok(config) => {
            let settings: Settings = config.try_into()?;
            Ok(settings)
        }
        Err(e) => Err(e),
    }
}

impl TryFrom<Config> for Settings {
    type Error = ConfigError;

    fn try_from(config: Config) -> Result<Self, Self::Error> {
        if config.get::<u16>("application_port").is_err()
            || config.get::<DatabaseSettings>("database").is_err()
        {
            return Err(ConfigError::Message(String::from("Not enough field")));
        }
        Ok(Self {
            application_port: config.get("application_port").unwrap(),
            database: config.get("database").unwrap(),
        })
    }
}
