//! Tauri 命令处理模块

use crate::frp::{ConfigManager, FrpConfig, FrpProcessManager, FrpVersionManager, FrpVersionInfo, ProcessState, validate_config};
use crate::utils::settings::{AppSettings, SettingsManager};
use log::{error, info};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use tauri::{Manager, State};
use tokio::sync::{mpsc, Mutex};

/// 应用状态
pub struct AppState {
    pub config_manager: Mutex<Option<ConfigManager>>,
    pub process_manager: Mutex<Option<FrpProcessManager>>,
    pub log_tx: Mutex<Option<mpsc::Sender<String>>>,
    pub settings_manager: Mutex<Option<SettingsManager>>,
    pub version_manager: Mutex<Option<FrpVersionManager>>,
}

impl AppState {
    pub fn new() -> Self {
        let (log_tx, _) = mpsc::channel(100);
        Self {
            config_manager: Mutex::new(None),
            process_manager: Mutex::new(None),
            log_tx: Mutex::new(Some(log_tx)),
            settings_manager: Mutex::new(None),
            version_manager: Mutex::new(None),
        }
    }

    pub fn with_config(config_manager: ConfigManager) -> Self {
        let (log_tx, _) = mpsc::channel(100);
        Self {
            config_manager: Mutex::new(Some(config_manager)),
            process_manager: Mutex::new(None),
            log_tx: Mutex::new(Some(log_tx)),
            settings_manager: Mutex::new(None),
            version_manager: Mutex::new(None),
        }
    }
}

// ==================== 设置持久化 ====================

#[tauri::command]
pub async fn load_settings(state: State<'_, AppState>) -> Result<AppSettings, String> {
    let sm = state.settings_manager.lock().await;
    match sm.as_ref() {
        Some(m) => m.load().map_err(|e| e.to_string()),
        None => Ok(AppSettings::default()),
    }
}

#[tauri::command]
pub async fn save_settings(settings: AppSettings, state: State<'_, AppState>) -> Result<bool, String> {
    info!("Saving settings: {:?}", settings);
    let sm = state.settings_manager.lock().await;
    match sm.as_ref() {
        Some(m) => {
            m.save(&settings).map_err(|e| e.to_string())?;
            Ok(true)
        }
        None => Err("设置管理器未初始化".to_string()),
    }
}

// ==================== 文件选择 ====================

#[tauri::command]
pub async fn pick_file(title: String, filters: Option<Vec<String>>) -> Result<Option<String>, String> {
    // 前端使用 @tauri-apps/plugin-dialog 直接调用，这里保留备用
    Ok(None)
}

#[tauri::command]
pub async fn pick_directory(title: String) -> Result<Option<String>, String> {
    Ok(None)
}

// ==================== 配置管理 ====================

#[derive(Debug, Serialize, Deserialize)]
pub struct ConfigResponse {
    pub success: bool,
    pub config: Option<FrpConfig>,
    pub error: Option<String>,
}

#[tauri::command]
pub async fn load_config(state: State<'_, AppState>) -> Result<ConfigResponse, String> {
    let cm = state.config_manager.lock().await;
    match cm.as_ref() {
        Some(m) => match m.load() {
            Ok(config) => Ok(ConfigResponse { success: true, config: Some(config), error: None }),
            Err(e) => Ok(ConfigResponse { success: false, config: None, error: Some(e.to_string()) }),
        },
        None => Ok(ConfigResponse { success: false, config: None, error: Some("配置管理器未初始化".into()) }),
    }
}

#[tauri::command]
pub async fn save_config(config: FrpConfig, state: State<'_, AppState>) -> Result<ConfigResponse, String> {
    if let Err(e) = validate_config(&config) {
        return Ok(ConfigResponse { success: false, config: None, error: Some(e) });
    }
    let cm = state.config_manager.lock().await;
    match cm.as_ref() {
        Some(m) => match m.save(&config) {
            Ok(_) => Ok(ConfigResponse { success: true, config: Some(config), error: None }),
            Err(e) => Ok(ConfigResponse { success: false, config: None, error: Some(e.to_string()) }),
        },
        None => Ok(ConfigResponse { success: false, config: None, error: Some("配置管理器未初始化".into()) }),
    }
}

#[tauri::command]
pub async fn export_config(target_path: String, state: State<'_, AppState>) -> Result<bool, String> {
    let cm = state.config_manager.lock().await;
    match cm.as_ref() {
        Some(m) => { m.export_to(&PathBuf::from(&target_path)).map_err(|e| e.to_string())?; Ok(true) }
        None => Err("配置管理器未初始化".into()),
    }
}

#[tauri::command]
pub async fn import_config(source_path: String, state: State<'_, AppState>) -> Result<FrpConfig, String> {
    let cm = state.config_manager.lock().await;
    match cm.as_ref() {
        Some(m) => { m.import_from(&PathBuf::from(&source_path)).map_err(|e| e.to_string())?; m.load().map_err(|e| e.to_string()) }
        None => Err("配置管理器未初始化".into()),
    }
}

// ==================== 进程控制 ====================

#[derive(Debug, Serialize, Deserialize)]
pub struct ProcessStatusResponse {
    pub running: bool,
    pub pid: Option<u32>,
    pub state: String,
}

#[tauri::command]
pub async fn start_frp(config: FrpConfig, state: State<'_, AppState>) -> Result<bool, String> {
    info!("Starting FRP");
    let mut pm_guard = state.process_manager.lock().await;

    #[cfg(windows)]
    let default_frpc = "frpc.exe";
    #[cfg(not(windows))]
    let default_frpc = "frpc";

    // 优先从版本管理器获取已下载的 frpc 路径
    let vm_guard = state.version_manager.lock().await;
    let frpc_path = vm_guard.as_ref()
        .and_then(|vm| vm.get_downloaded_frpc_path())
        .map(|p| p.to_string_lossy().to_string())
        .unwrap_or_else(|| {
            std::env::var("FRPC_PATH").unwrap_or_else(|_| default_frpc.to_string())
        });
    drop(vm_guard);

    let config_dir = dirs::config_dir()
        .map(|d| d.join("frpc-gui"))
        .unwrap_or_else(|| std::path::PathBuf::from("."));
    let config_path = config_dir.join("frpc.toml");

    let log_tx = {
        let tx_guard = state.log_tx.lock().await;
        match tx_guard.clone() {
            Some(tx) => tx,
            None => return Err("日志系统未初始化".to_string()),
        }
    };

    let mut pm = FrpProcessManager::new(
        std::path::PathBuf::from(&frpc_path),
        config_path,
        log_tx,
    );

    match pm.start(&config).await {
        Ok(_) => { *pm_guard = Some(pm); Ok(true) }
        Err(e) => { error!("Failed to start FRP: {}", e); Err(e.to_string()) }
    }
}

#[tauri::command]
pub async fn stop_frp(state: State<'_, AppState>) -> Result<bool, String> {
    let mut pm_guard = state.process_manager.lock().await;
    match pm_guard.as_mut() {
        Some(pm) => match pm.stop().await { Ok(_) => Ok(true), Err(e) => Err(e.to_string()) },
        None => Ok(true),
    }
}

#[tauri::command]
pub async fn restart_frp(config: FrpConfig, state: State<'_, AppState>) -> Result<bool, String> {
    let mut pm_guard = state.process_manager.lock().await;
    match pm_guard.as_mut() {
        Some(pm) => match pm.restart(&config).await { Ok(_) => Ok(true), Err(e) => Err(e.to_string()) },
        None => Err("FRP 进程未运行".to_string()),
    }
}

#[tauri::command]
pub async fn get_process_status(state: State<'_, AppState>) -> Result<ProcessStatusResponse, String> {
    let pm_guard = state.process_manager.lock().await;
    match pm_guard.as_ref() {
        Some(pm) => {
            let proc_state = pm.get_state();
            let (running, pid, state_str) = match proc_state {
                ProcessState::Running { pid } => (true, Some(pid), "running".to_string()),
                ProcessState::Starting => (false, None, "starting".to_string()),
                ProcessState::Stopping => (false, None, "stopping".to_string()),
                ProcessState::Stopped => (false, None, "stopped".to_string()),
                ProcessState::Error(e) => (false, None, format!("error: {}", e)),
            };
            Ok(ProcessStatusResponse { running, pid, state: state_str })
        }
        None => Ok(ProcessStatusResponse { running: false, pid: None, state: "not_initialized".to_string() }),
    }
}

// ==================== 日志 ====================

#[tauri::command]
pub async fn get_logs(_state: State<'_, AppState>) -> Result<Vec<String>, String> {
    Ok(vec![])
}

// ==================== FRP 版本管理 ====================

#[tauri::command]
pub async fn list_frp_versions(state: State<'_, AppState>) -> Result<Vec<FrpVersionInfo>, String> {
    let vm = state.version_manager.lock().await;
    match vm.as_ref() {
        Some(m) => m.list_versions().await.map_err(|e| e.to_string()),
        None => Err("版本管理器未初始化".to_string()),
    }
}

#[tauri::command]
pub async fn download_frp_version(version: String, url: String, state: State<'_, AppState>) -> Result<String, String> {
    info!("Downloading FRP {} from {}", version, url);
    let vm = state.version_manager.lock().await;
    match vm.as_ref() {
        Some(m) => {
            let path = m.download_version(&version, &url).await.map_err(|e| e.to_string())?;
            Ok(path.to_string_lossy().to_string())
        }
        None => Err("版本管理器未初始化".to_string()),
    }
}

#[tauri::command]
pub async fn delete_frp_version(version: String, state: State<'_, AppState>) -> Result<bool, String> {
    let vm = state.version_manager.lock().await;
    match vm.as_ref() {
        Some(m) => { m.delete_version(&version).map_err(|e| e.to_string())?; Ok(true) }
        None => Err("版本管理器未初始化".to_string()),
    }
}

#[tauri::command]
pub fn check_frpc_exists(path: String) -> bool {
    crate::frp::check_frpc_exists(&PathBuf::from(&path))
}

#[tauri::command]
pub fn get_frpc_version(path: String) -> Result<String, String> {
    crate::frp::get_frpc_version(&PathBuf::from(&path)).map_err(|e| e.to_string())
}

// ==================== 应用初始化 ====================

pub fn init_app(app: &mut tauri::App) {
    let config_dir = app.path().app_config_dir()
        .unwrap_or_else(|_| PathBuf::from("."))
        .join("frpc-gui");

    // 配置管理器
    let config_path = config_dir.join("frpc.toml");
    let config_manager = ConfigManager::new(config_path);

    // 设置管理器
    let settings_path = config_dir.join("settings.json");
    let settings_manager = SettingsManager::new(settings_path);

    // 版本管理器
    let install_dir = config_dir.join("bin");
    std::fs::create_dir_all(&install_dir).ok();
    let version_manager = FrpVersionManager::new(install_dir);

    let (log_tx, _) = mpsc::channel(100);
    let app_state = AppState {
        config_manager: Mutex::new(Some(config_manager)),
        process_manager: Mutex::new(None),
        log_tx: Mutex::new(Some(log_tx)),
        settings_manager: Mutex::new(Some(settings_manager)),
        version_manager: Mutex::new(Some(version_manager)),
    };
    app.manage(app_state);

    info!("Application initialized");
}
