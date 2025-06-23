use crate::{
    config::NetworkKind,
    error::{Error, Result},
    utils::image_to_tensor,
};
use candle_core::{DType, Device, Tensor};
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
    pub async fn new(kind: NetworkKind, device: &Device) -> Result<Self> {
        let config = kind.config();
        let model_name = kind.model_filename();
        let api = hf_hub::api::tokio::ApiBuilder::new()
            .with_cache_dir(std::path::PathBuf::from("./.cache"))
            .build()?;
        let api = api.model(model_name);
        let model_file = api.get("model.safetensors").await?;
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

    pub fn extract<T>(&self, image_path: T) -> Result<Vec<f32>>
    where
        T: AsRef<std::path::Path>,
    {
        let img = image_to_tensor(image_path, Some((self.resolution(), self.resolution())))?
            .to_device(&self.device)?;
        let feature = self.network.forward(&img.unsqueeze(0)?)?.flatten_all()?;
        Ok(feature.to_vec1::<f32>()?)
    }

    pub fn extract_batch<T>(&self, image_paths: &[T]) -> Result<Vec<Vec<f32>>>
    where
        T: AsRef<std::path::Path>,
    {
        let process_image = |path: &T| -> Result<Tensor> {
            Ok(
                image_to_tensor(path, Some((self.resolution(), self.resolution())))?
                    .to_device(&self.device)?,
            )
        };
        cfg_if::cfg_if! {
            if #[cfg(feature = "rayon")] {
                use rayon::prelude::*;
                let tensors = image_paths
                    .par_iter()
                    .map(process_image)
                    .collect::<Result<Vec<_>>>()?;
            } else {
                let tensors = image_paths
                    .iter()
                    .map(process_image)
                    .collect::<Result<Vec<_>>>()?;
            }
        };

        let batch_tensor = Tensor::stack(&tensors, 0)?;
        let features = self
            .network
            .forward(&batch_tensor)?
            .flatten_from(1)?
            .to_vec2::<f32>()?;

        Ok(features)
    }

    pub fn extract_folder<T>(&self, folder_path: T) -> Result<Vec<Vec<f32>>>
    where
        T: AsRef<std::path::Path>,
    {
        let path_str = folder_path.as_ref().to_string_lossy().to_string();
        if !(folder_path.as_ref().exists() && folder_path.as_ref().is_dir()) {
            return Err(Error::FolderNotFound(path_str));
        }
        let image_paths = std::fs::read_dir(folder_path)?
            .map(|entry| -> Result<_> { Ok(entry?.path()) })
            .collect::<Result<Vec<_>>>()?;
        if image_paths.is_empty() {
            return Err(Error::FolderEmpty(path_str));
        }
        self.extract_batch(&image_paths)
    }

    pub fn resolution(&self) -> u32 {
        self.kind.resolution()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_small() {
        let extractor = Extractor::new(NetworkKind::Small, &Device::Cpu)
            .await
            .unwrap();
        let feature = extractor.extract("data/cpp.png").unwrap();
        assert_eq!(feature.len(), FEATURE_SIZE);
    }

    #[tokio::test]
    async fn test_medium() {
        let extractor = Extractor::new(NetworkKind::Medium, &Device::Cpu)
            .await
            .unwrap();
        let feature = extractor.extract("data/cpp.png").unwrap();
        assert_eq!(feature.len(), FEATURE_SIZE);
    }

    #[tokio::test]
    async fn test_large() {
        let extractor = Extractor::new(NetworkKind::Large, &Device::Cpu)
            .await
            .unwrap();
        let feature = extractor.extract("data/cpp.png").unwrap();
        assert_eq!(feature.len(), FEATURE_SIZE);
    }

    #[tokio::test]
    async fn test_hybrid_medium() {
        let extractor = Extractor::new(NetworkKind::HybridMedium, &Device::Cpu)
            .await
            .unwrap();
        let feature = extractor.extract("data/cpp.png").unwrap();
        assert_eq!(feature.len(), FEATURE_SIZE);
    }

    #[tokio::test]
    async fn test_hybrid_large() {
        let extractor = Extractor::new(NetworkKind::HybridLarge, &Device::Cpu)
            .await
            .unwrap();
        let feature = extractor.extract("data/cpp.png").unwrap();
        assert_eq!(feature.len(), FEATURE_SIZE);
    }
}
