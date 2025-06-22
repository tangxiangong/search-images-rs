use crate::configration::Config;
use search_image::{
    App,
    config::{Device, MobilenetConfig},
    error::Error,
};

pub async fn get() -> App {
    let config = Config::load();
    let device = match config.mobilenet.device().into_device() {
        Ok(_) => config.mobilenet.device(),
        Err(e) => {
            tracing::warn!("Failed to use device: {:?}, using CPU instead", e);
            Device::Cpu
        }
    };
    let mobilenet_config = MobilenetConfig::new(config.mobilenet.kind(), device);
    match App::new(&config.db, &mobilenet_config).await {
        Ok(app) => app,
        Err(e) => {
            match e {
                Error::CUDAError => tracing::error!("Failed to use CUDA"),
                Error::MetalError => tracing::error!("Failed to use Metal"),
                Error::HuggingFaceApiError(e) => {
                    tracing::error!("Failed to use HuggingFace API: {}", e)
                }
                Error::QdrantBuildError(e) => {
                    tracing::error!("Failed to build Qdrant: {}", e)
                }
                Error::CandleError(e) => {
                    tracing::error!("Failed to use Candle: {}", e)
                }
                _ => tracing::error!("Failed to load app: {:?}", e),
            }
            std::process::exit(1);
        }
    }
}
