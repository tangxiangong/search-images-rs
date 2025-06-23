use std::sync::{Arc, Mutex};
use slint::ComponentHandle;
use crate::{AppState, MainWindow};

pub struct FileHandler;

impl FileHandler {
    pub fn setup_file_selection(main_window: &MainWindow, app_state: Arc<Mutex<AppState>>) {
        let main_window_weak = main_window.as_weak();
        main_window.on_select_files(move || {
            Self::handle_file_selection(app_state.clone(), main_window_weak.clone());
        });
    }

    pub fn setup_remove_image(main_window: &MainWindow, app_state: Arc<Mutex<AppState>>) {
        let main_window_weak = main_window.as_weak();
        main_window.on_remove_image(move |file_path| {
            Self::handle_remove_image(
                app_state.clone(), 
                main_window_weak.clone(), 
                file_path.to_string()
            );
        });
    }

    pub fn setup_clear_all(main_window: &MainWindow, app_state: Arc<Mutex<AppState>>) {
        let main_window_weak = main_window.as_weak();
        main_window.on_clear_all(move || {
            Self::handle_clear_all(app_state.clone(), main_window_weak.clone());
        });
    }

    fn handle_file_selection(
        app_state: Arc<Mutex<AppState>>, 
        main_window_weak: slint::Weak<MainWindow>
    ) {
        let files = rfd::FileDialog::new()
            .add_filter("图片文件", &["png", "jpg", "jpeg", "bmp", "tiff", "webp"])
            .set_title("选择图片文件")
            .pick_files();

        if let Some(files) = files {
            let mut state = app_state.lock().unwrap();
            for file in files {
                let image_data = crate::types::ImageData::new(file);
                state.add_image(image_data);
            }

            if let Some(window) = main_window_weak.upgrade() {
                super::ui_handler::UIHandler::update_ui_images(&state, &window);
            }
        }
    }

    fn handle_remove_image(
        app_state: Arc<Mutex<AppState>>, 
        main_window_weak: slint::Weak<MainWindow>,
        file_path: String
    ) {
        let mut state = app_state.lock().unwrap();
        state.remove_image(&file_path);
        if let Some(window) = main_window_weak.upgrade() {
            super::ui_handler::UIHandler::update_ui_images(&state, &window);
        }
    }

    fn handle_clear_all(
        app_state: Arc<Mutex<AppState>>, 
        main_window_weak: slint::Weak<MainWindow>
    ) {
        let mut state = app_state.lock().unwrap();
        state.clear_images();
        if let Some(window) = main_window_weak.upgrade() {
            super::ui_handler::UIHandler::update_ui_images(&state, &window);
        }
    }
}