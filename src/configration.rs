use search_images_core::config::{DbConfig, MobilenetConfig};
use serde::Deserialize;
use tokio::net::TcpListener;

#[derive(Debug, Clone, Deserialize)]
pub struct Config {
    pub port: u16,
    pub db: DbConfig,
    pub mobilenet: MobilenetConfig,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            port: 8080,
            db: DbConfig::default(),
            mobilenet: MobilenetConfig::default(),
        }
    }
}

impl Config {
    pub fn load() -> Self {
        match config::Config::builder()
            .add_source(config::File::with_name("config"))
            .build()
        {
            Ok(config) => config.try_deserialize::<Config>().unwrap_or_else(|e| {
                let default_config = Config::default();
                tracing::warn!(
                    "Failed to parse config: {}, using default config: {:?}",
                    e,
                    default_config
                );
                default_config
            }),
            Err(e) => {
                let default_config = Config::default();
                tracing::error!(
                    "Failed to load config: {}, using default config: {:?}",
                    e,
                    default_config
                );
                default_config
            }
        }
    }

    pub async fn tcp_listener(&self) -> TcpListener {
        TcpListener::bind(format!("127.0.0.1:{}", self.port))
            .await
            .unwrap_or_else(|_| panic!("Failed to bind to port {}", self.port))
    }
}
