use crate::{
    config::{DbConfig, MobilenetConfig},
    error::{Error, Result},
    extractor::{Extractor, FEATURE_SIZE},
};
use qdrant_client::{
    Qdrant, QdrantBuilder,
    config::CompressionEncoding,
    qdrant::{CreateCollectionBuilder, Distance, VectorParamsBuilder},
};

pub struct App {
    db: Qdrant,
    extractor: Extractor,
    collection: String,
}

impl App {
    pub async fn new(db_config: &DbConfig, mobilenet_config: &MobilenetConfig) -> Result<Self> {
        let device = mobilenet_config.device().into_device()?;
        let extractor = Extractor::new(mobilenet_config.kind(), &device)?;
        let qdrant_url = format!("http://{}:{}", db_config.url(), db_config.port());
        let db = QdrantBuilder::from_url(&qdrant_url)
            .connect_timeout(std::time::Duration::from_secs(30))
            .timeout(std::time::Duration::from_secs(30))
            .compression(Some(CompressionEncoding::Gzip))
            .keep_alive_while_idle()
            .build()
            .map_err(|e| Error::QdrantBuildError(e.to_string()))?;

        let collection = db_config.collection().to_string();

        if !db
            .collection_exists(&collection)
            .await
            .map_err(|e| Error::CollectionError(e.to_string()))?
        {
            db.create_collection(CreateCollectionBuilder::new(&collection).vectors_config(
                VectorParamsBuilder::new(FEATURE_SIZE as u64, Distance::Cosine),
            ))
            .await
            .map_err(|e| Error::CollectionError(e.to_string()))?;
        }

        Ok(Self {
            db,
            extractor,
            collection,
        })
    }

    pub fn db(&self) -> &Qdrant {
        &self.db
    }

    pub fn extractor(&self) -> &Extractor {
        &self.extractor
    }

    pub fn collection(&self) -> &str {
        &self.collection
    }
}
