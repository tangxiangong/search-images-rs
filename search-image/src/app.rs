use crate::{
    config::{DbConfig, MobilenetConfig},
    database,
    error::{Error, Result},
    extractor::{Extractor, FEATURE_SIZE},
};
use qdrant_client::{
    Qdrant, QdrantBuilder,
    config::CompressionEncoding,
    qdrant::{CreateCollectionBuilder, Distance, VectorParamsBuilder},
};
use serde::{Deserialize, Serialize, de::DeserializeOwned};

pub struct App {
    db: Qdrant,
    extractor: Extractor,
    collection: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImageInfo<T = ()> {
    #[serde(skip)]
    id: String,
    path: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    extra: Option<T>,
}

impl<T> ImageInfo<T> {
    pub fn with_extra(path: &str, extra: T) -> Self {
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            path: path.to_string(),
            extra: Some(extra),
        }
    }

    pub fn with_path(path: &str) -> Self {
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            path: path.to_string(),
            extra: None,
        }
    }

    pub fn id(&self) -> &str {
        &self.id
    }

    pub fn path(&self) -> &str {
        &self.path
    }

    pub fn extra(&self) -> Option<&T> {
        self.extra.as_ref()
    }
}

impl App {
    pub async fn new(db_config: &DbConfig, mobilenet_config: &MobilenetConfig) -> Result<Self> {
        let device = mobilenet_config.device().into_device()?;
        let extractor = Extractor::new(mobilenet_config.kind(), &device).await?;
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

    pub async fn add_images<T: AsRef<std::path::Path>>(&self, paths: &[T]) -> Result<()> {
        let info = paths
            .iter()
            .map(|path| ImageInfo::with_path(&path.as_ref().to_string_lossy()))
            .collect::<Vec<ImageInfo<()>>>();
        let features = self.extractor().extract_batch(paths)?;
        database::add(&self.db, &self.collection, &features, &info).await
    }

    pub async fn add_images_with_extra<
        T: Serialize + DeserializeOwned + Clone,
        P: AsRef<std::path::Path>,
    >(
        &self,
        paths: &[P],
        extras: &[T],
    ) -> Result<()> {
        let info = paths
            .iter()
            .zip(extras)
            .map(|(path, extra)| {
                ImageInfo::with_extra(&path.as_ref().to_string_lossy(), extra.to_owned())
            })
            .collect::<Vec<ImageInfo<T>>>();
        let features = self.extractor().extract_batch(paths)?;
        database::add(&self.db, &self.collection, &features, &info).await
    }
}
