// 在 lib.rs 中集中包含 Slint 生成的模块
slint::include_modules!();

// 导出模块
pub mod app_state;
pub mod extractor_async;
pub mod handlers;
pub mod types;

// 重新导出主要类型
pub use app_state::AppState;
pub use extractor_async::AsyncExtractor;
pub use handlers::{FeatureHandler, FileHandler, UIHandler};
pub use types::ImageData;
