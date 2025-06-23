use anyhow::Result;
use slint::ComponentHandle;
use slint_ui::{
    AppState, MainWindow, ModelStatus,
    handlers::{FeatureHandler, FileHandler, UIHandler},
};
use std::sync::{Arc, Mutex};

#[tokio::main]
async fn main() -> Result<()> {
    let main_window = MainWindow::new()?;
    let app_state = Arc::new(Mutex::new(AppState::new()));

    // 初始化模型状态，包含默认选中的模型
    let selected_model = {
        let state = app_state.lock().unwrap();
        state.get_model_string()
    };

    main_window.set_model_status(ModelStatus {
        loaded: false,
        loading: false,
        downloading: false,
        download_progress: 0.0,
        model_type: "".into(),
        selected_model: selected_model.into(),
    });

    setup_event_handlers(&main_window, app_state.clone());
    FeatureHandler::start_model_loading(app_state.clone(), main_window.as_weak());

    main_window.run()?;
    Ok(())
}

fn setup_event_handlers(main_window: &MainWindow, app_state: Arc<Mutex<AppState>>) {
    FileHandler::setup_file_selection(main_window, app_state.clone());
    FeatureHandler::setup_feature_extraction(main_window, app_state.clone());
    FeatureHandler::setup_toggle_features(main_window, app_state.clone());
    FeatureHandler::setup_extract_all_features(main_window, app_state.clone());
    UIHandler::setup_remove_image(main_window, app_state.clone());
    UIHandler::setup_clear_all(main_window, app_state.clone());
    setup_model_change(main_window, app_state.clone());
}

fn setup_model_change(main_window: &MainWindow, app_state: Arc<Mutex<AppState>>) {
    let main_window_weak = main_window.as_weak();
    main_window.on_change_model(move |model_str| {
        let model_string = model_str.to_string();
        let app_state_inner = app_state.clone();
        let main_window_inner = main_window_weak.clone();

        // 更新选中的模型
        {
            let mut state = app_state_inner.lock().unwrap();
            let network_kind = AppState::parse_model_from_string(&model_string);
            state.set_selected_model(network_kind);
        }

        // 更新UI显示选中的模型
        if let Some(window) = main_window_inner.upgrade() {
            let state = app_state_inner.lock().unwrap();
            window.set_model_status(ModelStatus {
                loaded: false,
                loading: false,
                downloading: false,
                download_progress: 0.0,
                model_type: "".into(),
                selected_model: state.get_model_string().into(),
            });
        }

        // 重新加载模型
        FeatureHandler::start_model_loading(app_state_inner, main_window_inner);
    });
}
