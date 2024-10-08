use config::{Config, ConfigError};
use serde::Deserialize;
// use std::convert::TryInto;

#[derive(Debug, Deserialize)]
pub struct DatabaseSettings {
    pub url: String,
    pub test_url: String,
}

#[derive(Debug, Deserialize)]
pub struct RedisSettings {
    pub uri: String,
}

#[derive(Debug, Deserialize)]
pub struct JwtSettings {
    pub secret: String,
}

#[derive(Debug, Deserialize)]
pub struct Settings {
    pub database: DatabaseSettings,
    pub redis: RedisSettings,
    pub jwt: JwtSettings,
}

impl Settings {
    pub fn new() -> Result<Self, ConfigError> {
        let mut s = Config::default();
        s.merge(config::File::with_name("config"))?;
        s.try_into()
    }
}
