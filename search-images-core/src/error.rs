#[allow(clippy::result_large_err)]
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
}

pub type Result<T> = std::result::Result<T, Error>;
