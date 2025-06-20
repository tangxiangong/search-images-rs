use crate::{error::AppResult, utils::image_to_tensor};
use candle_core::{DType, Device};
use candle_nn::{Module, VarBuilder};
use candle_transformers::models::{
    mimi::candle_nn::Func,
    mobilenetv4::{self, Config as MobileNetV4Config},
};

pub const FEATURE_SIZE: usize = 960;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Kind {
    Small,
    Medium,
    Large,
    HybridMedium,
    HybridLarge,
}

impl Kind {
    fn model_filename(&self) -> String {
        let name = match self {
            Self::Small => "conv_small.e2400_r224",
            Self::Medium => "conv_medium.e500_r256",
            Self::HybridMedium => "hybrid_medium.ix_e550_r256",
            Self::Large => "conv_large.e600_r384",
            Self::HybridLarge => "hybrid_large.ix_e600_r384",
        };
        format!("timm/mobilenetv4_{}_in1k", name)
    }

    pub fn resolution(&self) -> u32 {
        match self {
            Self::Small => 224,
            Self::Medium => 256,
            Self::HybridMedium => 256,
            Self::Large => 384,
            Self::HybridLarge => 384,
        }
    }

    fn config(&self) -> MobileNetV4Config {
        match self {
            Self::Small => MobileNetV4Config::small(),
            Self::Medium => MobileNetV4Config::medium(),
            Self::HybridMedium => MobileNetV4Config::hybrid_medium(),
            Self::Large => MobileNetV4Config::large(),
            Self::HybridLarge => MobileNetV4Config::hybrid_large(),
        }
    }
}

pub struct Extractor {
    kind: Kind,
    network: Func<'static>,
    device: Device,
}

impl Extractor {
    pub fn new(kind: Kind, device: Device) -> AppResult<Self> {
        let config = kind.config();
        let model_name = kind.model_filename();
        let api = hf_hub::api::sync::ApiBuilder::new()
            .with_cache_dir(std::path::PathBuf::from("./.cache"))
            .build()?;
        let api = api.model(model_name);
        let model_file = api.get("model.safetensors")?;
        let vb =
            unsafe { VarBuilder::from_mmaped_safetensors(&[model_file], DType::F32, &device)? };
        let network = mobilenetv4::mobilenetv4_no_final_layer(&config, vb)?;
        Ok(Self {
            kind,
            network,
            device,
        })
    }

    pub fn kind(&self) -> Kind {
        self.kind
    }

    pub fn config(&self) -> MobileNetV4Config {
        self.kind.config()
    }

    pub fn extract(&self, image_path: impl AsRef<std::path::Path>) -> AppResult<Vec<f32>> {
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
        let extractor = Extractor::new(Kind::Small, Device::Cpu).unwrap();
        let feature = extractor.extract("data/cpp.png").unwrap();
        assert_eq!(feature.len(), FEATURE_SIZE);
    }

    #[test]
    fn test_medium() {
        let extractor = Extractor::new(Kind::Medium, Device::Cpu).unwrap();
        let feature = extractor.extract("data/cpp.png").unwrap();
        assert_eq!(feature.len(), FEATURE_SIZE);
    }

    #[test]
    fn test_large() {
        let extractor = Extractor::new(Kind::Large, Device::Cpu).unwrap();
        let feature = extractor.extract("data/cpp.png").unwrap();
        assert_eq!(feature.len(), FEATURE_SIZE);
    }

    #[test]
    fn test_hybrid_medium() {
        let extractor = Extractor::new(Kind::HybridMedium, Device::Cpu).unwrap();
        let feature = extractor.extract("data/cpp.png").unwrap();
        assert_eq!(feature.len(), FEATURE_SIZE);
    }

    #[test]
    fn test_hybrid_large() {
        let extractor = Extractor::new(Kind::HybridLarge, Device::Cpu).unwrap();
        let feature = extractor.extract("data/cpp.png").unwrap();
        assert_eq!(feature.len(), FEATURE_SIZE);
    }
}
