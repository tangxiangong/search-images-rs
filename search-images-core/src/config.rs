use crate::error::{Error, Result};
use candle_transformers::models::mobilenetv4;
use serde::Deserialize;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum NetworkKind {
    Small,
    Medium,
    Large,
    HybridMedium,
    #[default]
    HybridLarge,
}

impl NetworkKind {
    pub fn model_filename(&self) -> String {
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

    pub(crate) fn config(&self) -> mobilenetv4::Config {
        match self {
            Self::Small => mobilenetv4::Config::small(),
            Self::Medium => mobilenetv4::Config::medium(),
            Self::HybridMedium => mobilenetv4::Config::hybrid_medium(),
            Self::Large => mobilenetv4::Config::large(),
            Self::HybridLarge => mobilenetv4::Config::hybrid_large(),
        }
    }
}

#[derive(Debug, Clone, Deserialize)]
pub struct DbConfig {
    url: String,
    port: u16,
    collection: String,
}

impl DbConfig {
    pub fn url(&self) -> &str {
        &self.url
    }

    pub fn port(&self) -> u16 {
        self.port
    }

    pub fn collection(&self) -> &str {
        &self.collection
    }
}

#[derive(Debug, Clone, Default, Deserialize)]
pub struct MobilenetConfig {
    kind: NetworkKind,
    device: Device,
}

impl MobilenetConfig {
    pub fn new(kind: NetworkKind, device: Device) -> Self {
        Self { kind, device }
    }

    pub fn kind(&self) -> NetworkKind {
        self.kind
    }

    pub fn device(&self) -> Device {
        self.device
    }
}

#[derive(Debug, Copy, Clone, Default, PartialEq, Eq, Deserialize)]
pub enum Device {
    #[default]
    Cpu,
    Gpu,
    Metal,
}

impl Device {
    pub fn into_device(&self) -> Result<candle_core::Device> {
        match self {
            Device::Cpu => Ok(candle_core::Device::Cpu),
            Device::Gpu => {
                if let Ok(cuda) = candle_core::Device::new_cuda(0) {
                    Ok(cuda)
                } else {
                    Err(Error::CUDAError)
                }
            }
            Device::Metal => {
                if let Ok(metal) = candle_core::Device::new_metal(0) {
                    Ok(metal)
                } else {
                    Err(Error::MetalError)
                }
            }
        }
    }
}

impl Default for DbConfig {
    fn default() -> Self {
        Self {
            url: "127.0.0.1".to_string(),
            port: 6333,
            collection: "images".to_string(),
        }
    }
}
