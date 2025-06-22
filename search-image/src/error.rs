#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("CUDA not available")]
    CUDAError,
    #[error("Metal not available")]
    MetalError,
    #[error("HuggingFace API Error: {0}")]
    HuggingFaceApiError(#[from] hf_hub::api::sync::ApiError),
    #[error("Candle Error: {0}")]
    CandleError(#[from] candle_core::Error),
    #[error("IO Error: {0}")]
    IOError(#[from] std::io::Error),
    #[error("Image Error: {0}")]
    ImageError(#[from] image::ImageError),
    #[error("Qdrant Error: {0}")]
    QdrantBuildError(String),
    #[error("Folder not found: {0}")]
    FolderNotFound(String),
    #[error("Folder is empty: {0}")]
    FolderEmpty(String),
    #[error("Collection Error: {0}")]
    CollectionError(String),
    #[error("Serde Error: {0}")]
    SerdeError(#[from] serde_json::Error),
    #[error("Json to Payload Error: {0}")]
    JsonToPayloadError(String),
    #[error("Upsert Points Error: {0}")]
    UpsertPointsError(String),
    #[error("Delete Points Error: {0}")]
    DeletePointsError(String),
    #[error("Search Points Error: {0}")]
    SearchPointsError(String),
}

pub type Result<T> = std::result::Result<T, Error>;
