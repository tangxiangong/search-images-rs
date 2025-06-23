use crate::{AppState, MainWindow, types::ImageData};
use slint::{ComponentHandle, ModelRc, VecModel};
use std::sync::{Arc, Mutex};

pub struct UIHandler;

impl UIHandler {
    pub fn update_ui_images(state: &AppState, window: &MainWindow) {
        let ui_images: Vec<crate::ImageItem> =
            state.images.iter().map(|img| img.to_image_item()).collect();

        let model = ModelRc::new(VecModel::from(ui_images));
        window.set_uploaded_images(model);
    }

    #[allow(dead_code)]
    fn get_features_preview(img: &ImageData) -> String {
        if let Some(ref features) = img.features {
            let preview_count = std::cmp::min(10, features.len());
            let preview: Vec<String> = features
                .iter()
                .take(preview_count)
                .map(|f| format!("{:.3}", f))
                .collect();

            if features.len() > preview_count {
                format!("[{}...] ({}个特征)", preview.join(", "), features.len())
            } else {
                format!("[{}]", preview.join(", "))
            }
        } else {
            "未提取".to_string()
        }
    }

    pub fn setup_remove_image(main_window: &MainWindow, app_state: Arc<Mutex<AppState>>) {
        let main_window_weak = main_window.as_weak();
        main_window.on_remove_image(move |file_path| {
            let file_path_str = file_path.to_string();
            let mut state = app_state.lock().unwrap();
            state.remove_image(&file_path_str);
            if let Some(window) = main_window_weak.upgrade() {
                Self::update_ui_images(&state, &window);
            }
        });
    }

    pub fn setup_clear_all(main_window: &MainWindow, app_state: Arc<Mutex<AppState>>) {
        let main_window_weak = main_window.as_weak();
        main_window.on_clear_all(move || {
            let mut state = app_state.lock().unwrap();
            state.clear_images();
            if let Some(window) = main_window_weak.upgrade() {
                Self::update_ui_images(&state, &window);
            }
        });
    }
}
