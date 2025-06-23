use crate::{AppState, AsyncExtractor, MainWindow, ModelStatus};
use slint::ComponentHandle;
use std::sync::{Arc, Mutex};

pub struct FeatureHandler;

impl FeatureHandler {
    pub fn setup_feature_extraction(main_window: &MainWindow, app_state: Arc<Mutex<AppState>>) {
        let main_window_weak = main_window.as_weak();
        main_window.on_extract_features(move |file_path| {
            let file_path_str = file_path.to_string();
            let app_state_inner = app_state.clone();
            let main_window_inner = main_window_weak.clone();

            slint::spawn_local(async move {
                Self::extract_single_feature(app_state_inner, main_window_inner, file_path_str)
                    .await;
            })
            .unwrap();
        });
    }

    pub fn setup_toggle_features(main_window: &MainWindow, app_state: Arc<Mutex<AppState>>) {
        let main_window_weak = main_window.as_weak();
        main_window.on_toggle_features(move |file_path| {
            Self::handle_toggle_features(
                app_state.clone(),
                main_window_weak.clone(),
                file_path.to_string(),
            );
        });
    }

    pub fn setup_extract_all_features(main_window: &MainWindow, app_state: Arc<Mutex<AppState>>) {
        let main_window_weak = main_window.as_weak();
        main_window.on_extract_all_features(move || {
            let app_state_inner = app_state.clone();
            let main_window_inner = main_window_weak.clone();

            slint::spawn_local(async move {
                Self::handle_extract_all_features(app_state_inner, main_window_inner).await;
            })
            .unwrap();
        });
    }

    pub fn start_model_loading(
        app_state: Arc<Mutex<AppState>>,
        main_window_weak: slint::Weak<MainWindow>,
    ) {
        slint::spawn_local(async move {
            Self::handle_model_loading(app_state, main_window_weak).await;
        })
        .unwrap();
    }

    async fn extract_single_feature(
        app_state: Arc<Mutex<AppState>>,
        main_window_weak: slint::Weak<MainWindow>,
        file_path: String,
    ) {
        // 检查模型是否可用
        let extractor_available = {
            let state = app_state.lock().unwrap();
            state.extractor.is_some()
        };

        if !extractor_available {
            // 更新错误状态
            {
                let mut state = app_state.lock().unwrap();
                if let Some(img) = state.find_image_mut(&file_path) {
                    img.error_message = "模型未加载".to_string();
                    img.processing = false;
                }
            }
            if let Some(window) = main_window_weak.upgrade() {
                let state = app_state.lock().unwrap();
                super::ui_handler::UIHandler::update_ui_images(&state, &window);
            }
            return;
        }

        // 设置处理中状态
        {
            let mut state = app_state.lock().unwrap();
            if let Some(img) = state.find_image_mut(&file_path) {
                img.processing = true;
                img.error_message.clear();
            }
        }

        // 更新UI显示处理中状态
        if let Some(window) = main_window_weak.upgrade() {
            let state = app_state.lock().unwrap();
            super::ui_handler::UIHandler::update_ui_images(&state, &window);
        }

        let result = std::thread::spawn({
            let app_state = app_state.clone();
            let file_path = file_path.clone();
            move || {
                // 获取必要信息
                let (extractor_available, img_path, already_extracted) = {
                    let state = app_state.lock().unwrap();
                    if let Some(img) = state
                        .images
                        .iter()
                        .find(|img| img.file_path.to_string_lossy() == file_path)
                    {
                        (
                            state.extractor.is_some(),
                            img.file_path.clone(),
                            img.features_extracted,
                        )
                    } else {
                        return Err(anyhow::anyhow!("图片未找到"));
                    }
                };

                if !extractor_available {
                    return Err(anyhow::anyhow!("模型未加载"));
                }

                if already_extracted {
                    return Ok(());
                }

                // 提取特征
                let features = {
                    let state = app_state.lock().unwrap();
                    if let Some(ref extractor) = state.extractor {
                        extractor
                            .extract(&img_path)
                            .map_err(|e| anyhow::anyhow!("提取失败: {}", e))
                    } else {
                        Err(anyhow::anyhow!("模型未加载"))
                    }
                }?;

                // 更新状态
                {
                    let mut state = app_state.lock().unwrap();
                    if let Some(img) = state.find_image_mut(&file_path) {
                        img.features = Some(features);
                        img.features_extracted = true;
                        img.processing = false;
                        img.error_message.clear();
                    }
                }

                Ok(())
            }
        })
        .join();

        // 更新最终状态
        match result {
            Ok(Ok(())) => {
                // 成功提取特征
                if let Some(window) = main_window_weak.upgrade() {
                    let state = app_state.lock().unwrap();
                    super::ui_handler::UIHandler::update_ui_images(&state, &window);
                }
            }
            Ok(Err(e)) => {
                // 提取失败
                {
                    let mut state = app_state.lock().unwrap();
                    if let Some(img) = state.find_image_mut(&file_path) {
                        img.processing = false;
                        img.error_message = e.to_string();
                        img.features_extracted = false;
                    }
                }
                if let Some(window) = main_window_weak.upgrade() {
                    let state = app_state.lock().unwrap();
                    super::ui_handler::UIHandler::update_ui_images(&state, &window);
                }
                eprintln!("特征提取失败: {}", e);
            }
            Err(_) => {
                // 线程执行失败
                {
                    let mut state = app_state.lock().unwrap();
                    if let Some(img) = state.find_image_mut(&file_path) {
                        img.processing = false;
                        img.error_message = "线程执行失败".to_string();
                        img.features_extracted = false;
                    }
                }
                if let Some(window) = main_window_weak.upgrade() {
                    let state = app_state.lock().unwrap();
                    super::ui_handler::UIHandler::update_ui_images(&state, &window);
                }
                eprintln!("线程执行失败");
            }
        }
    }

    async fn handle_model_loading(
        app_state: Arc<Mutex<AppState>>,
        main_window_weak: slint::Weak<MainWindow>,
    ) {
        let selected_model = {
            let state = app_state.lock().unwrap();
            state.get_selected_model()
        };

        // 使用异步扩展器加载模型
        match AsyncExtractor::load_model_with_smart_progress(
            selected_model,
            app_state.clone(),
            main_window_weak.clone(),
        )
        .await
        {
            Ok(extractor) => {
                // 成功加载模型
                {
                    let mut state = app_state.lock().unwrap();
                    state.extractor = Some(extractor);
                    state.model_loading = false;
                }

                if let Some(main_window) = main_window_weak.upgrade() {
                    let model_name = {
                        let state = app_state.lock().unwrap();
                        state.get_model_display_name().to_string()
                    };
                    slint::invoke_from_event_loop({
                        let main_window_weak = main_window_weak.clone();
                        let app_state = app_state.clone();
                        move || {
                            if let Some(window) = main_window_weak.upgrade() {
                                let state = app_state.lock().unwrap();
                                window.set_model_status(ModelStatus {
                                    loaded: true,
                                    loading: false,
                                    downloading: false,
                                    download_progress: 1.0,
                                    model_type: model_name.into(),
                                    selected_model: state.get_model_string().into(),
                                });
                            }
                        }
                    })
                    .unwrap();
                }
            }
            Err(e) => {
                // 加载失败
                {
                    let mut state = app_state.lock().unwrap();
                    state.model_loading = false;
                }

                if let Some(main_window) = main_window_weak.upgrade() {
                    slint::invoke_from_event_loop({
                        let main_window_weak = main_window_weak.clone();
                        let app_state = app_state.clone();
                        let error_msg = e.to_string();
                        move || {
                            if let Some(window) = main_window_weak.upgrade() {
                                let state = app_state.lock().unwrap();
                                window.set_model_status(ModelStatus {
                                    loaded: false,
                                    loading: false,
                                    downloading: false,
                                    download_progress: 0.0,
                                    model_type: format!("加载失败: {}", error_msg).into(),
                                    selected_model: state.get_model_string().into(),
                                });
                            }
                        }
                    })
                    .unwrap();
                }
            }
        }
    }

    fn handle_toggle_features(
        app_state: Arc<Mutex<AppState>>,
        main_window_weak: slint::Weak<MainWindow>,
        file_path: String,
    ) {
        let mut state = app_state.lock().unwrap();
        if let Some(img) = state.find_image_mut(&file_path) {
            img.show_features = !img.show_features;
        }
        if let Some(window) = main_window_weak.upgrade() {
            super::ui_handler::UIHandler::update_ui_images(&state, &window);
        }
    }

    async fn handle_extract_all_features(
        app_state: Arc<Mutex<AppState>>,
        main_window_weak: slint::Weak<MainWindow>,
    ) {
        let file_paths: Vec<String> = {
            let state = app_state.lock().unwrap();
            state.get_unprocessed_images()
        };

        for file_path in file_paths {
            Self::extract_single_feature(app_state.clone(), main_window_weak.clone(), file_path)
                .await;
        }
    }
}
