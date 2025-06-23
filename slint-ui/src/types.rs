use crate::ImageItem;
use std::path::PathBuf;

//slint::include_modules!();

#[derive(Clone, Debug)]
pub struct ImageData {
    pub file_path: PathBuf,
    pub file_name: String,
    pub file_size: String,
    pub processing: bool,
    pub features_extracted: bool,
    pub error_message: String,
    pub features: Option<Vec<f32>>,
    pub show_features: bool,
}

impl ImageData {
    pub fn new(file_path: PathBuf) -> Self {
        let file_name = file_path
            .file_name()
            .unwrap_or_default()
            .to_string_lossy()
            .to_string();

        let file_size = match std::fs::metadata(&file_path) {
            Ok(metadata) => format_file_size(metadata.len()),
            Err(_) => "未知".to_string(),
        };

        Self {
            file_path,
            file_name,
            file_size,
            processing: false,
            features_extracted: false,
            error_message: String::new(),
            features: None,
            show_features: false,
        }
    }

    pub fn to_image_item(&self) -> ImageItem {
        let features_preview = if self.show_features && self.features.is_some() {
            let features = self.features.as_ref().unwrap();
            let preview_count = std::cmp::min(30, features.len());

            // 格式化特征向量，每行显示10个值
            let mut lines = Vec::new();
            for chunk in features[..preview_count].chunks(10) {
                let line: Vec<String> = chunk.iter().map(|f| format!("{:7.4}", f)).collect();
                lines.push(format!("  [{}]", line.join(", ")));
            }

            let mut result = format!("特征维度: {} \n", features.len());
            result.push_str(&lines.join("\n"));

            if features.len() > preview_count {
                result.push_str(&format!(
                    "\n  ... (还有 {} 个特征值)",
                    features.len() - preview_count
                ));
            }

            // 添加一些统计信息
            let min_val = features.iter().fold(f32::INFINITY, |acc, &x| acc.min(x));
            let max_val = features
                .iter()
                .fold(f32::NEG_INFINITY, |acc, &x| acc.max(x));
            let mean_val = features.iter().sum::<f32>() / features.len() as f32;

            result.push_str(&format!("\n\n统计信息:"));
            result.push_str(&format!("\n  最小值: {:7.4}", min_val));
            result.push_str(&format!("\n  最大值: {:7.4}", max_val));
            result.push_str(&format!("\n  平均值: {:7.4}", mean_val));

            result
        } else {
            String::new()
        };

        ImageItem {
            file_path: self.file_path.to_string_lossy().to_string().into(),
            file_name: self.file_name.clone().into(),
            file_size: self.file_size.clone().into(),
            preview_image: Default::default(), // TODO: 加载实际图片预览
            processing: self.processing,
            features_extracted: self.features_extracted,
            error_message: self.error_message.clone().into(),
            features_preview: features_preview.into(),
            show_features: self.show_features,
        }
    }
}

fn format_file_size(size: u64) -> String {
    const UNITS: &[&str] = &["B", "KB", "MB", "GB"];
    let mut size = size as f64;
    let mut unit_index = 0;

    while size >= 1024.0 && unit_index < UNITS.len() - 1 {
        size /= 1024.0;
        unit_index += 1;
    }

    format!("{:.1} {}", size, UNITS[unit_index])
}
