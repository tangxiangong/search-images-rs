use crate::{config::NetworkKind, error::Result, utils::image_to_tensor};
use candle_core::{DType, Device};
use candle_nn::{Module, VarBuilder};
use candle_transformers::models::{mimi::candle_nn::Func, mobilenetv4};

pub const FEATURE_SIZE: usize = 960;

#[derive(Debug, Clone)]
pub struct Extractor {
    kind: NetworkKind,
    network: Func<'static>,
    device: Device,
}

impl Extractor {
    pub fn new(kind: NetworkKind, device: &Device) -> Result<Self> {
        let config = kind.config();
        let model_name = kind.model_filename();
        let api = hf_hub::api::sync::ApiBuilder::new()
            .with_cache_dir(std::path::PathBuf::from("./.cache"))
            .build()?;
        let api = api.model(model_name);
        let model_file = api.get("model.safetensors")?;
        let vb = unsafe { VarBuilder::from_mmaped_safetensors(&[model_file], DType::F32, device)? };
        let network = mobilenetv4::mobilenetv4_no_final_layer(&config, vb)?;
        Ok(Self {
            kind,
            network,
            device: device.clone(),
        })
    }

    pub fn kind(&self) -> NetworkKind {
        self.kind
    }

    pub fn config(&self) -> mobilenetv4::Config {
        self.kind.config()
    }

    pub fn extract(&self, image_path: impl AsRef<std::path::Path>) -> Result<Vec<f32>> {
        let img = image_to_tensor(image_path, Some((self.resolution(), self.resolution())))?
            .to_device(&self.device)?;
        let feature = self.network.forward(&img.unsqueeze(0)?)?.flatten_all()?;
        Ok(feature.to_vec1::<f32>()?)
    }

    pub fn resolution(&self) -> u32 {
        self.kind.resolution()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_small() {
        let extractor = Extractor::new(NetworkKind::Small, &Device::Cpu).unwrap();
        let feature = extractor.extract("data/cpp.png").unwrap();
        assert_eq!(feature.len(), FEATURE_SIZE);
    }

    #[test]
    fn test_medium() {
        let extractor = Extractor::new(NetworkKind::Medium, &Device::Cpu).unwrap();
        let feature = extractor.extract("data/cpp.png").unwrap();
        assert_eq!(feature.len(), FEATURE_SIZE);
    }

    #[test]
    fn test_large() {
        let extractor = Extractor::new(NetworkKind::Large, &Device::Cpu).unwrap();
        let feature = extractor.extract("data/cpp.png").unwrap();
        assert_eq!(feature.len(), FEATURE_SIZE);
    }

    #[test]
    fn test_hybrid_medium() {
        let extractor = Extractor::new(NetworkKind::HybridMedium, &Device::Cpu).unwrap();
        let feature = extractor.extract("data/cpp.png").unwrap();
        assert_eq!(feature.len(), FEATURE_SIZE);
    }

    #[test]
    fn test_hybrid_large() {
        let extractor = Extractor::new(NetworkKind::HybridLarge, &Device::Cpu).unwrap();
        let feature = extractor.extract("data/cpp.png").unwrap();
        assert_eq!(feature.len(), FEATURE_SIZE);
    }
}
