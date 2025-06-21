use crate::{
    config::{DbConfig, MobilenetConfig},
    error::{Error, Result},
    extractor::Extractor,
};
use qdrant_client::{Qdrant, QdrantBuilder, config::CompressionEncoding};

pub struct App {
    db: Qdrant,
    extractor: Extractor,
}

impl App {
    pub fn new(db_config: &DbConfig, mobilenet_config: &MobilenetConfig) -> Result<Self> {
        let device = mobilenet_config.device.into_device()?;
        let extractor = Extractor::new(mobilenet_config.kind, &device)?;
        let qdrant_url = format!("http://{}:{}", &db_config.url, db_config.port);
        let db = QdrantBuilder::from_url(&qdrant_url)
            .connect_timeout(std::time::Duration::from_secs(30))
            .timeout(std::time::Duration::from_secs(30))
            .compression(Some(CompressionEncoding::Gzip))
            .keep_alive_while_idle()
            .build()
            .map_err(|e| Error::QdrantBuildError(e.to_string()))?;

        Ok(Self { db, extractor })
    }

    pub fn db(&self) -> &Qdrant {
        &self.db
    }

    pub fn extractor(&self) -> &Extractor {
        &self.extractor
    }
}
