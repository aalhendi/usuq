use figment::{
    providers::{Env, Format, Toml},
    Figment,
};
use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct Settings {
    pub database: DatabaseSettings,
    pub application: ApplicationSettings,
}
#[derive(serde::Deserialize, Debug)]
pub struct ApplicationSettings {
    pub port: u16,
    pub host: String,
}

#[derive(Deserialize, Debug)]
pub struct DatabaseSettings {
    pub path: String,
}

impl Settings {
    pub fn new() -> Result<Settings, figment::Error> {
        let base_path =
            std::env::current_dir().expect("Failed to determine the current directory.");
        let configutation_directory = base_path.join("configuration");

        let environment: Environment = std::env::var("APP_ENVIRONMENT")
            .unwrap_or_else(|_| String::from("local"))
            .try_into()
            .expect("Failed to parse APP_ENVIRONMENT");
        let environment_filename =
            format!("{environment}.toml", environment = environment.as_str());

        let config = Figment::new()
            // Start with default
            .merge(Toml::file(configutation_directory.join("base.toml")))
            // Layer on environment-specific
            .merge(Toml::file(
                configutation_directory.join(environment_filename),
            ))
            // Layer on environment variables
            .merge(Env::prefixed("APP_").split("__"));

        config.extract()
    }
}

/// The possible runtime environment for our application.
pub enum Environment {
    Local,
    Production,
}

impl Environment {
    pub fn as_str(&self) -> &'static str {
        match self {
            Environment::Local => "local",
            Environment::Production => "production",
        }
    }
}

impl TryFrom<String> for Environment {
    type Error = String;
    fn try_from(s: String) -> Result<Self, Self::Error> {
        match s.to_lowercase().as_str() {
            "local" => Ok(Self::Local),
            "production" => Ok(Self::Production),
            other => Err(format!(
                "{other} is not a supported environment. Use either `local` or `production`.",
            )),
        }
    }
}
