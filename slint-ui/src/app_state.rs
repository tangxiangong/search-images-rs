use crate::types::ImageData;
use search_image::{config::NetworkKind, extractor::Extractor};

pub struct AppState {
    pub images: Vec<ImageData>,
    pub extractor: Option<Extractor>,
    pub model_loading: bool,
    pub selected_model: NetworkKind,
}

impl Default for AppState {
    fn default() -> Self {
        Self::new()
    }
}

impl AppState {
    pub fn new() -> Self {
        Self {
            images: Vec::new(),
            extractor: None,
            model_loading: false,
            selected_model: NetworkKind::HybridLarge, // 默认选择
        }
    }

    pub fn add_image(&mut self, image_data: ImageData) {
        self.images.push(image_data);
    }

    pub fn find_image_mut(&mut self, file_path: &str) -> Option<&mut ImageData> {
        self.images
            .iter_mut()
            .find(|img| img.file_path.to_string_lossy() == file_path)
    }

    pub fn remove_image(&mut self, file_path: &str) {
        self.images
            .retain(|img| img.file_path.to_string_lossy() != file_path);
    }

    pub fn clear_images(&mut self) {
        self.images.clear();
    }

    pub fn get_unprocessed_images(&self) -> Vec<String> {
        self.images
            .iter()
            .filter(|img| !img.features_extracted && !img.processing)
            .map(|img| img.file_path.to_string_lossy().to_string())
            .collect()
    }

    pub fn set_selected_model(&mut self, model: NetworkKind) {
        self.selected_model = model;
        // 切换模型时清除当前加载的提取器
        self.extractor = None;
    }

    pub fn get_selected_model(&self) -> NetworkKind {
        self.selected_model
    }

    pub fn get_model_display_name(&self) -> &'static str {
        match self.selected_model {
            NetworkKind::Small => "MobileNetV4-Small",
            NetworkKind::Medium => "MobileNetV4-Medium",
            NetworkKind::Large => "MobileNetV4-Large",
            NetworkKind::HybridMedium => "MobileNetV4-HybridMedium",
            NetworkKind::HybridLarge => "MobileNetV4-HybridLarge",
        }
    }

    pub fn parse_model_from_string(model_str: &str) -> NetworkKind {
        match model_str {
            "Small" => NetworkKind::Small,
            "Medium" => NetworkKind::Medium,
            "Large" => NetworkKind::Large,
            "HybridMedium" => NetworkKind::HybridMedium,
            "HybridLarge" => NetworkKind::HybridLarge,
            _ => NetworkKind::HybridLarge, // 默认值
        }
    }

    pub fn get_model_string(&self) -> String {
        match self.selected_model {
            NetworkKind::Small => "Small".to_string(),
            NetworkKind::Medium => "Medium".to_string(),
            NetworkKind::Large => "Large".to_string(),
            NetworkKind::HybridMedium => "HybridMedium".to_string(),
            NetworkKind::HybridLarge => "HybridLarge".to_string(),
        }
    }
}
