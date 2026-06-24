use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct Config {
    pub server: ServerConfig,
    pub dal: DalConfig,
    pub clsi: ClsiConfig,
}

#[derive(Debug, Deserialize)]
pub struct ServerConfig {
    pub host: String,
    pub port: u16,
}

#[derive(Debug, Deserialize)]
pub struct DalConfig {
    pub addr: String,
}

#[derive(Debug, Deserialize)]
pub struct ClsiConfig {
    pub addr: String,
}

impl Config {
    pub fn load() -> Result<Self, config::ConfigError> {
        let env = std::env::var("APP_ENV").unwrap_or_else(|_| "development".into());

        let cfg = config::Config::builder()
            .set_default("server.host", "0.0.0.0")?
            .set_default("server.port", 3000)?
            .set_default("dal.addr", "http://127.0.0.1:4000")?
            .set_default("clsi.addr", "http://127.0.0.1:5001")?
            .add_source(config::File::with_name("config/default"))
            .add_source(config::File::with_name(&format!("config/{env}")).required(false))
            .add_source(
                config::Environment::with_prefix("APP")
                    .separator("_")
                    .try_parsing(true),
            )
            .build()?;

        cfg.try_deserialize()
    }

    pub fn server_addr(&self) -> String {
        format!("{}:{}", self.server.host, self.server.port)
    }
}

