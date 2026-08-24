//! Tauri 命令处理模块

use crate::frp::{ConfigManager, FrpConfig, FrpProcessManager, ProcessState, validate_config};
use log::{error, info};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::sync::Arc;
use tauri::{AppHandle, Manager, State};
use tokio::sync::{mpsc, Mutex};

/// 应用状态
pub struct AppState {
    pub config_manager: Mutex<Option<ConfigManager>>,
    pub process_manager: Mutex<Option<FrpProcessManager>>,
    pub log_tx: Mutex<Option<mpsc::Sender<String>>>,
}

impl AppState {
    pub fn new() -> Self {
        let (log_tx, _) = mpsc::channel(100);
        Self {
            config_manager: Mutex::new(None),
            process_manager: Mutex::new(None),
            log_tx: Mutex::new(Some(log_tx)),
        }
    }
}

/// 配置相关命令
#[derive(Debug, Serialize, Deserialize)]
pub struct ConfigResponse {
    pub success: bool,
    pub config: Option<FrpConfig>,
    pub error: Option<String>,
}

#[tauri::command]
pub async fn load_config(state: State<'_, AppState>) -> Result<ConfigResponse, String> {
    let config_manager = state.config_manager.lock().await;
    
    match config_manager.as_ref() {
        Some(cm) => match cm.load() {
            Ok(config) => Ok(ConfigResponse {
                success: true,
                config: Some(config),
                error: None,
            }),
            Err(e) => Ok(ConfigResponse {
                success: false,
                config: None,
                error: Some(e.to_string()),
            }),
        },
        None => Ok(ConfigResponse {
            success: false,
            config: None,
            error: Some("配置管理器未初始化".to_string()),
        }),
    }
}

#[tauri::command]
pub async fn save_config(config: FrpConfig, state: State<'_, AppState>) -> Result<ConfigResponse, String> {
    // 验证配置
    if let Err(e) = validate_config(&config) {
        return Ok(ConfigResponse {
            success: false,
            config: None,
            error: Some(e),
        });
    }

    let config_manager = state.config_manager.lock().await;
    
    match config_manager.as_ref() {
        Some(cm) => match cm.save(&config) {
            Ok(_) => Ok(ConfigResponse {
                success: true,
                config: Some(config),
                error: None,
            }),
            Err(e) => Ok(ConfigResponse {
                success: false,
                config: None,
                error: Some(e.to_string()),
            }),
        },
        None => Ok(ConfigResponse {
            success: false,
            config: None,
            error: Some("配置管理器未初始化".to_string()),
        }),
    }
}

#[tauri::command]
pub async fn export_config(
    target_path: String,
    state: State<'_, AppState>,
) -> Result<bool, String> {
    let config_manager = state.config_manager.lock().await;
    
    match config_manager.as_ref() {
        Some(cm) => {
            cm.export_to(&PathBuf::from(&target_path))
                .map_err(|e| e.to_string())?;
            Ok(true)
        }
        None => Err("配置管理器未初始化".to_string()),
    }
}

#[tauri::command]
pub async fn import_config(
    source_path: String,
    state: State<'_, AppState>,
) -> Result<FrpConfig, String> {
    let config_manager = state.config_manager.lock().await;
    
    match config_manager.as_ref() {
        Some(cm) => {
            cm.import_from(&PathBuf::from(&source_path))
                .map_err(|e| e.to_string())?;
            cm.load().map_err(|e| e.to_string())
        }
        None => Err("配置管理器未初始化".to_string()),
    }
}

/// 进程控制命令
#[derive(Debug, Serialize, Deserialize)]
pub struct ProcessStatusResponse {
    pub running: bool,
    pub pid: Option<u32>,
    pub state: String,
}

#[tauri::command]
pub async fn start_frp(config: FrpConfig, state: State<'_, AppState>) -> Result<bool, String> {
    info!("Starting FRP with config: {:?}", config);
    
    let mut process_manager = state.process_manager.lock().await;
    
    // 获取 frpc 路径
    let frpc_path = PathBuf::from("frpc"); // TODO: 从应用配置获取
    let config_path = PathBuf::from("frpc.toml"); // TODO: 从应用配置获取
    
    let log_tx = state.log_tx.lock().await.clone().unwrap();
    
    let mut pm = FrpProcessManager::new(frpc_path, config_path, log_tx);
    
    match pm.start(&config).await {
        Ok(_) => {
            *process_manager = Some(pm);
            Ok(true)
        }
        Err(e) => {
            error!("Failed to start FRP: {}", e);
            Err(e.to_string())
        }
    }
}

#[tauri::command]
pub async fn stop_frp(state: State<'_, AppState>) -> Result<bool, String> {
    info!("Stopping FRP");
    
    let mut process_manager = state.process_manager.lock().await;
    
    match process_manager.as_mut() {
        Some(pm) => match pm.stop().await {
            Ok(_) => Ok(true),
            Err(e) => Err(e.to_string()),
        },
        None => Ok(true), // 未在运行
    }
}

#[tauri::command]
pub async fn restart_frp(config: FrpConfig, state: State<'_, AppState>) -> Result<bool, String> {
    info!("Restarting FRP");
    
    let mut process_manager = state.process_manager.lock().await;
    
    match process_manager.as_mut() {
        Some(pm) => match pm.restart(&config).await {
            Ok(_) => Ok(true),
            Err(e) => Err(e.to_string()),
        },
        None => Err("FRP 进程未运行".to_string()),
    }
}

#[tauri::command]
pub async fn get_process_status(state: State<'_, AppState>) -> Result<ProcessStatusResponse, String> {
    let process_manager = state.process_manager.lock().await;
    
    match process_manager.as_ref() {
        Some(pm) => {
            let state = pm.get_state();
            let (running, pid, state_str) = match state {
                ProcessState::Running { pid } => (true, Some(pid), "running".to_string()),
                ProcessState::Starting => (false, None, "starting".to_string()),
                ProcessState::Stopping => (false, None, "stopping".to_string()),
                ProcessState::Stopped => (false, None, "stopped".to_string()),
                ProcessState::Error(e) => (false, None, format!("error: {}", e)),
            };
            
            Ok(ProcessStatusResponse {
                running,
                pid,
                state: state_str,
            })
        }
        None => Ok(ProcessStatusResponse {
            running: false,
            pid: None,
            state: "not_initialized".to_string(),
        }),
    }
}

/// 日志相关命令
#[tauri::command]
pub async fn get_logs(_state: State<'_, AppState>) -> Result<Vec<String>, String> {
    // TODO: 实现日志历史记录
    Ok(vec![])
}

/// 系统相关命令
#[tauri::command]
pub fn check_frpc_exists(path: String) -> bool {
    crate::frp::check_frpc_exists(&PathBuf::from(&path))
}

#[tauri::command]
pub fn get_frpc_version(path: String) -> Result<String, String> {
    crate::frp::get_frpc_version(&PathBuf::from(&path)).map_err(|e| e.to_string())
}

/// 应用初始化
pub fn init_app(app: &mut tauri::App) {
    // 初始化应用状态
    let app_state = AppState::new();
    app.manage(app_state);
    
    // 设置配置管理器路径
    let config_path = app
        .path()
        .app_config_dir()
        .unwrap_or_else(|_| PathBuf::from("."))
        .join("frpc.toml");
    
    let config_manager = ConfigManager::new(config_path);
    
    // 更新状态
    let state = app.state::<AppState>();
    let rt = tokio::runtime::Handle::current();
    rt.block_on(async {
        *state.config_manager.lock().await = Some(config_manager);
    });
    
    info!("Application initialized");
}
