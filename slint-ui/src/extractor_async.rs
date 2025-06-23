use crate::{AppState, MainWindow, ModelStatus};
use search_image::{config::NetworkKind, extractor::Extractor};
use std::sync::{Arc, Mutex};
use std::time::Duration;
use tokio::time::sleep;

pub struct AsyncExtractor;

impl AsyncExtractor {
    /// 异步加载模型，带有进度模拟
    pub async fn load_model_with_progress(
        network_kind: NetworkKind,
        app_state: Arc<Mutex<AppState>>,
        main_window_weak: slint::Weak<MainWindow>,
    ) -> Result<Extractor, Box<dyn std::error::Error + Send + Sync>> {
        // 模拟下载进度
        Self::simulate_download_progress(main_window_weak.clone(), app_state.clone()).await;

        // 切换到加载状态
        if let Some(window) = main_window_weak.upgrade() {
            let state = app_state.lock().unwrap();
            let model_string = state.get_model_string();
            drop(state);
            slint::invoke_from_event_loop({
                let main_window_weak = main_window_weak.clone();
                move || {
                    if let Some(window) = main_window_weak.upgrade() {
                        window.set_model_status(ModelStatus {
                            loaded: false,
                            loading: true,
                            downloading: false,
                            download_progress: 1.0,
                            model_type: "".into(),
                            selected_model: model_string.into(),
                        });
                    }
                }
            })
            .unwrap();
        }

        // 实际加载模型
        let device = candle_core::Device::Cpu;
        let extractor = Extractor::new(network_kind, &device).await?;

        Ok(extractor)
    }

    /// 模拟下载进度
    async fn simulate_download_progress(
        main_window_weak: slint::Weak<MainWindow>,
        app_state: Arc<Mutex<AppState>>,
    ) {
        let steps = 20; // 20步完成下载
        let delay = Duration::from_millis(100); // 每步100ms

        for i in 0..=steps {
            let progress = i as f32 / steps as f32;

            if let Some(window) = main_window_weak.upgrade() {
                let state = app_state.lock().unwrap();
                let model_string = state.get_model_string();
                drop(state);

                slint::invoke_from_event_loop({
                    let main_window_weak = main_window_weak.clone();
                    let model_string = model_string.clone();
                    move || {
                        if let Some(window) = main_window_weak.upgrade() {
                            window.set_model_status(ModelStatus {
                                loaded: false,
                                loading: false,
                                downloading: true,
                                download_progress: progress,
                                model_type: "".into(),
                                selected_model: model_string.into(),
                            });
                        }
                    }
                })
                .unwrap();
            }

            sleep(delay).await;
        }
    }

    /// 检查模型文件是否已经缓存
    pub fn is_model_cached(network_kind: NetworkKind) -> bool {
        let model_name = network_kind.model_filename();
        let cache_dir = std::path::PathBuf::from("./.cache")
            .join("huggingface")
            .join("hub")
            .join(format!("models--{}", model_name.replace("/", "--")));

        // 检查是否存在模型文件
        if cache_dir.exists() {
            // 简单检查是否有 model.safetensors 文件
            let mut found_model = false;
            if let Ok(entries) = std::fs::read_dir(&cache_dir) {
                for entry in entries.flatten() {
                    if entry.path().is_dir() {
                        let snapshot_dir = entry.path();
                        if let Ok(snapshot_entries) = std::fs::read_dir(&snapshot_dir) {
                            for snapshot_entry in snapshot_entries.flatten() {
                                let model_file = snapshot_entry.path().join("model.safetensors");
                                if model_file.exists() {
                                    found_model = true;
                                    break;
                                }
                            }
                        }
                    }
                    if found_model {
                        break;
                    }
                }
            }
            found_model
        } else {
            false
        }
    }

    /// 获取模型大小估计（用于更准确的进度显示）
    pub fn get_model_size_estimate(network_kind: NetworkKind) -> u64 {
        match network_kind {
            NetworkKind::Small => 50 * 1024 * 1024,         // ~50MB
            NetworkKind::Medium => 100 * 1024 * 1024,       // ~100MB
            NetworkKind::Large => 200 * 1024 * 1024,        // ~200MB
            NetworkKind::HybridMedium => 150 * 1024 * 1024, // ~150MB
            NetworkKind::HybridLarge => 300 * 1024 * 1024,  // ~300MB
        }
    }

    /// 获取下载时间估计（基于网络速度和模型大小）
    pub fn get_download_duration_estimate(network_kind: NetworkKind) -> Duration {
        let size_mb = Self::get_model_size_estimate(network_kind) / (1024 * 1024);
        let assumed_speed_mbps = 10.0; // 假设10MB/s的下载速度
        let seconds = size_mb as f64 / assumed_speed_mbps;
        Duration::from_secs(seconds as u64)
    }

    /// 带有智能进度的模型加载
    pub async fn load_model_with_smart_progress(
        network_kind: NetworkKind,
        app_state: Arc<Mutex<AppState>>,
        main_window_weak: slint::Weak<MainWindow>,
    ) -> Result<Extractor, Box<dyn std::error::Error + Send + Sync>> {
        // 检查是否已缓存
        let is_cached = Self::is_model_cached(network_kind);

        if is_cached {
            // 如果已缓存，直接进入加载状态
            if let Some(window) = main_window_weak.upgrade() {
                let state = app_state.lock().unwrap();
                let model_string = state.get_model_string();
                drop(state);
                slint::invoke_from_event_loop({
                    let main_window_weak = main_window_weak.clone();
                    move || {
                        if let Some(window) = main_window_weak.upgrade() {
                            window.set_model_status(ModelStatus {
                                loaded: false,
                                loading: true,
                                downloading: false,
                                download_progress: 1.0,
                                model_type: "从缓存加载中...".into(),
                                selected_model: model_string.into(),
                            });
                        }
                    }
                })
                .unwrap();
            }
        } else {
            // 如果未缓存，显示下载进度
            let duration = Self::get_download_duration_estimate(network_kind);
            Self::simulate_realistic_download_progress(
                main_window_weak.clone(),
                app_state.clone(),
                duration,
            )
            .await;

            // 切换到加载状态
            if let Some(window) = main_window_weak.upgrade() {
                let state = app_state.lock().unwrap();
                let model_string = state.get_model_string();
                drop(state);
                slint::invoke_from_event_loop({
                    let main_window_weak = main_window_weak.clone();
                    move || {
                        if let Some(window) = main_window_weak.upgrade() {
                            window.set_model_status(ModelStatus {
                                loaded: false,
                                loading: true,
                                downloading: false,
                                download_progress: 1.0,
                                model_type: "初始化模型中...".into(),
                                selected_model: model_string.into(),
                            });
                        }
                    }
                })
                .unwrap();
            }
        }

        // 实际加载模型
        let device = candle_core::Device::Cpu;
        let extractor = Extractor::new(network_kind, &device).await?;

        Ok(extractor)
    }

    /// 更真实的下载进度模拟
    async fn simulate_realistic_download_progress(
        main_window_weak: slint::Weak<MainWindow>,
        app_state: Arc<Mutex<AppState>>,
        total_duration: Duration,
    ) {
        let steps = 50; // 更多步骤，更平滑的进度
        let step_duration = total_duration / steps;

        for i in 0..=steps {
            // 模拟真实的下载曲线（开始快，中间稳定，最后可能稍慢）
            let raw_progress = i as f32 / steps as f32;
            let progress = if raw_progress < 0.1 {
                raw_progress * 2.0 // 前10%下载较快
            } else if raw_progress < 0.9 {
                0.2 + (raw_progress - 0.1) * 0.7 / 0.8 // 中间80%稳定下载
            } else {
                0.9 + (raw_progress - 0.9) * 0.1 / 0.1 // 最后10%稍慢
            };

            if let Some(window) = main_window_weak.upgrade() {
                let state = app_state.lock().unwrap();
                let model_string = state.get_model_string();
                let model_display_name = state.get_model_display_name();
                drop(state);

                slint::invoke_from_event_loop({
                    let main_window_weak = main_window_weak.clone();
                    let model_string = model_string.clone();
                    let model_display_name = model_display_name.to_string();
                    move || {
                        if let Some(window) = main_window_weak.upgrade() {
                            window.set_model_status(ModelStatus {
                                loaded: false,
                                loading: false,
                                downloading: true,
                                download_progress: progress,
                                model_type: format!("下载 {}", model_display_name).into(),
                                selected_model: model_string.into(),
                            });
                        }
                    }
                })
                .unwrap();
            }

            sleep(step_duration).await;
        }
    }
}
