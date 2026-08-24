//! Tauri 命令处理模块

use crate::frp::{
    ConfigManager, FrpConfig, FrpProcessManager, FrpVersionInfo, FrpVersionManager, MirrorInfo,
    ProcessState, validate_config,
};
use crate::utils::settings::{AppSettings, SettingsManager};
use log::{error, info};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use tauri::{Manager, State};
use tokio::sync::{mpsc, Mutex};

// ==================== 应用状态 ====================

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
    info!("Saving settings");
    let sm = state.settings_manager.lock().await;
    match sm.as_ref() {
        Some(m) => { m.save(&settings).map_err(|e| e.to_string())?; Ok(true) }
        None => Err("设置管理器未初始化".to_string()),
    }
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

/// 从 frpc.toml 导入配置
#[tauri::command]
pub async fn import_toml_config(toml_path: String, state: State<'_, AppState>) -> Result<FrpConfig, String> {
    info!("Importing TOML config from {}", toml_path);
    let cm = state.config_manager.lock().await;
    match cm.as_ref() {
        Some(m) => m.import_toml(&PathBuf::from(&toml_path)).map_err(|e| e.to_string()),
        None => Err("配置管理器未初始化".to_string()),
    }
}

// ==================== 进程控制 ====================

#[derive(Debug, Serialize, Deserialize)]
pub struct ProcessStatusResponse {
    pub running: bool,
    pub pid: Option<u32>,
    pub state: String,
    pub connection_error: Option<String>,
    pub last_start_time: Option<i64>,
}

#[tauri::command]
pub async fn start_frp(config: FrpConfig, state: State<'_, AppState>) -> Result<bool, String> {
    info!("Starting FRP");
    let mut pm_guard = state.process_manager.lock().await;

    // 获取 frpc 路径
    #[cfg(windows)]
    let default_frpc = "frpc.exe";
    #[cfg(not(windows))]
    let default_frpc = "frpc";

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

    // 启动前检查端口
    if config.web_server.port > 0 {
        if !FrpProcessManager::check_port_available(config.web_server.port) {
            return Err(format!("端口 {} 已被占用", config.web_server.port));
        }
    }

    match pm.start(&config).await {
        Ok(_) => {
            *pm_guard = Some(pm);
            Ok(true)
        }
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

/// 热重载配置（不重启进程）
#[tauri::command]
pub async fn reload_frp(config: FrpConfig, state: State<'_, AppState>) -> Result<bool, String> {
    let pm_guard = state.process_manager.lock().await;
    match pm_guard.as_ref() {
        Some(pm) => match pm.reload(&config).await { Ok(_) => Ok(true), Err(e) => Err(e.to_string()) },
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
            let connection_error = if running { pm.check_connection_error() } else { None };
            Ok(ProcessStatusResponse { running, pid, state: state_str, connection_error, last_start_time: None })
        }
        None => Ok(ProcessStatusResponse { running: false, pid: None, state: "not_initialized".to_string(), connection_error: None, last_start_time: None }),
    }
}

/// 检测外部 frpc 进程（应用重启后恢复）
#[tauri::command]
pub async fn detect_frpc_process(state: State<'_, AppState>) -> Result<bool, String> {
    let mut pm_guard = state.process_manager.lock().await;
    match pm_guard.as_mut() {
        Some(pm) => Ok(pm.detect_external_process()),
        None => Ok(false),
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
            let path = m.download_version(&version, &url, None).await.map_err(|e| e.to_string())?;
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

/// 获取可用镜像源列表
#[tauri::command]
pub fn get_mirrors() -> Vec<MirrorInfo> {
    crate::frp::download::get_mirrors()
}

/// 导入本地 frpc 文件
#[tauri::command]
pub async fn import_local_frpc(file_path: String, state: State<'_, AppState>) -> Result<String, String> {
    info!("Importing local frpc: {}", file_path);
    let vm = state.version_manager.lock().await;
    match vm.as_ref() {
        Some(m) => {
            let path = m.import_local_frpc(&PathBuf::from(&file_path)).map_err(|e| e.to_string())?;
            Ok(path.to_string_lossy().to_string())
        }
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

// ==================== 缺失功能补齐 ====================

/// #1 一键清空所有配置、下载、日志
#[tauri::command]
pub async fn reset_all_config(state: State<'_, AppState>) -> Result<bool, String> {
    info!("Resetting all config");

    // 1. 停止 FRP 进程
    let mut pm_guard = state.process_manager.lock().await;
    if let Some(pm) = pm_guard.as_mut() {
        let _ = pm.stop().await;
    }
    drop(pm_guard);

    // 2. 获取配置目录
    let config_dir = dirs::config_dir()
        .map(|d| d.join("frpc-gui"))
        .unwrap_or_else(|| PathBuf::from("."));

    // 3. 清空配置文件
    let config_path = config_dir.join("frpc.json");
    if config_path.exists() {
        std::fs::remove_file(&config_path).map_err(|e| e.to_string())?;
    }

    // 4. 清空下载目录
    let bin_dir = config_dir.join("bin");
    if bin_dir.exists() {
        std::fs::remove_dir_all(&bin_dir).map_err(|e| e.to_string())?;
        std::fs::create_dir_all(&bin_dir).ok();
    }

    // 5. 清空日志文件
    let log_path = config_dir.join("frpc.log");
    if log_path.exists() {
        std::fs::remove_file(&log_path).ok();
    }

    // 6. 重置内存状态
    let mut cm_guard = state.config_manager.lock().await;
    *cm_guard = Some(ConfigManager::new(config_dir.join("frpc.json")));
    drop(cm_guard);

    info!("All config reset complete");
    Ok(true)
}

/// #2 读取 frpc 日志文件内容
#[tauri::command]
pub async fn get_frpc_log_content(state: State<'_, AppState>) -> Result<String, String> {
    let config_dir = dirs::config_dir()
        .map(|d| d.join("frpc-gui"))
        .unwrap_or_else(|| PathBuf::from("."));
    let log_path = config_dir.join("frpc.log");

    if !log_path.exists() {
        return Ok(String::new());
    }

    std::fs::read_to_string(&log_path).map_err(|e| e.to_string())
}

/// #3 读取应用自身日志
#[tauri::command]
pub async fn get_app_log_content() -> Result<String, String> {
    // 返回内存中的最近日志
    // 实际应用日志通过 env_logger 输出到 stderr
    Ok(String::new())
}

/// #4 在文件管理器中打开日志文件
#[tauri::command]
pub async fn open_frpc_log_file() -> Result<bool, String> {
    let config_dir = dirs::config_dir()
        .map(|d| d.join("frpc-gui"))
        .unwrap_or_else(|| PathBuf::from("."));
    let log_path = config_dir.join("frpc.log");

    if !log_path.exists() {
        return Err("日志文件不存在".to_string());
    }

    #[cfg(windows)]
    {
        std::process::Command::new("explorer")
            .arg(&log_path)
            .spawn()
            .map_err(|e| e.to_string())?;
    }
    #[cfg(target_os = "macos")]
    {
        std::process::Command::new("open")
            .arg(&log_path)
            .spawn()
            .map_err(|e| e.to_string())?;
    }
    #[cfg(all(unix, not(target_os = "macos")))]
    {
        std::process::Command::new("xdg-open")
            .arg(&log_path)
            .spawn()
            .map_err(|e| e.to_string())?;
    }

    Ok(true)
}

/// #5 获取本地监听端口列表
#[tauri::command]
pub async fn get_local_ports() -> Result<Vec<LocalPort>, String> {
    info!("Getting local ports");

    #[cfg(windows)]
    {
        let output = std::process::Command::new("netstat")
            .args(["-a", "-n"])
            .output()
            .map_err(|e| e.to_string())?;

        let stdout = String::from_utf8_lossy(&output.stdout);
        let mut ports: Vec<LocalPort> = Vec::new();

        for line in stdout.lines() {
            if !line.contains("TCP") && !line.contains("UDP") {
                continue;
            }
            let cols: Vec<&str> = line.split_whitespace().collect();
            if cols.len() < 2 {
                continue;
            }
            let local = cols[1];
            if let Some(idx) = local.rfind(':') {
                let ip = &local[..idx];
                if let Ok(port) = local[idx + 1..].parse::<u16>() {
                    ports.push(LocalPort {
                        protocol: cols[0].to_string(),
                        ip: ip.to_string(),
                        port,
                    });
                }
            }
        }

        ports.sort_by_key(|p| p.port);
        ports.dedup_by_key(|p| (p.protocol.clone(), p.port));
        return Ok(ports);
    }

    #[cfg(unix)]
    {
        let output = std::process::Command::new("sh")
            .args(["-c", "netstat -an | grep LISTEN"])
            .output()
            .map_err(|e| e.to_string())?;

        let stdout = String::from_utf8_lossy(&output.stdout);
        let mut ports: Vec<LocalPort> = Vec::new();

        for line in stdout.lines() {
            let cols: Vec<&str> = line.split_whitespace().collect();
            if cols.len() < 4 {
                continue;
            }
            let local = cols[3];
            if let Some(idx) = local.rfind(if cfg!(target_os = "macos") { '.' } else { ':' }) {
                let ip = &local[..idx];
                if let Ok(port) = local[idx + 1..].parse::<u16>() {
                    let proto = cols[0].to_lowercase();
                    ports.push(LocalPort {
                        protocol: proto,
                        ip: ip.to_string(),
                        port,
                    });
                }
            }
        }

        ports.sort_by_key(|p| p.port);
        ports.dedup_by_key(|p| (p.protocol.clone(), p.port));
        return Ok(ports);
    }

    #[cfg(not(any(windows, unix)))]
    {
        Ok(vec![])
    }
}

/// #6 打开外部 URL
#[tauri::command]
pub async fn open_url(url: String) -> Result<bool, String> {
    info!("Opening URL: {}", url);

    #[cfg(windows)]
    {
        std::process::Command::new("rundll32")
            .args(["url.dll,FileProtocolHandler", &url])
            .spawn()
            .map_err(|e| e.to_string())?;
    }
    #[cfg(target_os = "macos")]
    {
        std::process::Command::new("open").arg(&url).spawn().map_err(|e| e.to_string())?;
    }
    #[cfg(all(unix, not(target_os = "macos")))]
    {
        std::process::Command::new("xdg-open").arg(&url).spawn().map_err(|e| e.to_string())?;
    }

    Ok(true)
}

/// #7 重启应用
#[tauri::command]
pub async fn relaunch_app(app: tauri::AppHandle) -> Result<bool, String> {
    info!("Relaunching app");
    app.restart();
    Ok(true)
}

/// #8 打开应用数据目录
#[tauri::command]
pub async fn open_app_data() -> Result<bool, String> {
    let config_dir = dirs::config_dir()
        .map(|d| d.join("frpc-gui"))
        .unwrap_or_else(|| PathBuf::from("."));

    if !config_dir.exists() {
        std::fs::create_dir_all(&config_dir).map_err(|e| e.to_string())?;
    }

    #[cfg(windows)]
    {
        std::process::Command::new("explorer")
            .arg(&config_dir)
            .spawn()
            .map_err(|e| e.to_string())?;
    }
    #[cfg(target_os = "macos")]
    {
        std::process::Command::new("open").arg(&config_dir).spawn().map_err(|e| e.to_string())?;
    }
    #[cfg(all(unix, not(target_os = "macos")))]
    {
        std::process::Command::new("xdg-open").arg(&config_dir).spawn().map_err(|e| e.to_string())?;
    }

    Ok(true)
}

/// #9 通用文件选择对话框
#[tauri::command]
pub async fn select_local_file(
    name: Option<String>,
    extensions: Option<Vec<String>>,
) -> Result<Option<String>, String> {
    // 前端已使用 @tauri-apps/plugin-dialog 直接调用
    // 此命令保留作为后端备用
    Ok(None)
}

/// #10 获取已下载版本列表
#[tauri::command]
pub async fn get_downloaded_versions(state: State<'_, AppState>) -> Result<Vec<FrpVersionInfo>, String> {
    let vm = state.version_manager.lock().await;
    match vm.as_ref() {
        Some(m) => {
            // 从磁盘扫描已下载的版本
            let install_dir = dirs::config_dir()
                .map(|d| d.join("frpc-gui").join("bin"))
                .unwrap_or_else(|| PathBuf::from("."));

            let mut versions = Vec::new();
            if let Ok(entries) = std::fs::read_dir(&install_dir) {
                for entry in entries.flatten() {
                    let path = entry.path();
                    if path.is_dir() {
                        let version_str = path.file_name()
                            .map(|n| n.to_string_lossy().to_string())
                            .unwrap_or_default();
                        let frpc_path = m.get_downloaded_frpc_path();
                        if frpc_path.is_some() {
                            versions.push(FrpVersionInfo {
                                version: version_str.clone(),
                                name: version_str.clone(),
                                published_at: String::new(),
                                download_url: String::new(),
                                mirror_url: None,
                                size: 0,
                                download_count: 0,
                                downloaded: true,
                                local_path: frpc_path.map(|p| p.to_string_lossy().to_string()),
                            });
                        }
                    }
                }
            }
            Ok(versions)
        }
        None => Err("版本管理器未初始化".to_string()),
    }
}

/// #11 修改代理状态（单独切换，触发热重载）
#[tauri::command]
pub async fn modify_proxy_status(
    proxy_name: String,
    enabled: bool,
    state: State<'_, AppState>,
) -> Result<bool, String> {
    info!("Modifying proxy status: {} -> {}", proxy_name, enabled);

    // 更新配置中的代理状态
    let cm = state.config_manager.lock().await;
    if let Some(m) = cm.as_ref() {
        let mut config = m.load().map_err(|e| e.to_string())?;
        if let Some(proxy) = config.proxies.iter_mut().find(|p| p.name == proxy_name) {
            proxy.enabled = enabled;
        }
        m.save(&config).map_err(|e| e.to_string())?;
    }
    drop(cm);

    // 尝试热重载
    let pm = state.process_manager.lock().await;
    if let Some(pm) = pm.as_ref() {
        let cm2 = state.config_manager.lock().await;
        if let Some(m) = cm2.as_ref() {
            if let Ok(config) = m.load() {
                let _ = pm.reload(&config).await;
            }
        }
    }

    Ok(true)
}

/// #12 检查应用更新（获取最新版本）
#[tauri::command]
pub async fn check_app_update() -> Result<String, String> {
    info!("Checking for app updates");

    let client = reqwest::Client::builder()
        .user_agent("frpc-gui")
        .timeout(std::time::Duration::from_secs(10))
        .build()
        .map_err(|e| e.to_string())?;

    let resp = client
        .get("https://api.github.com/repos/ccy-max/frpc-gui/releases/latest")
        .send()
        .await
        .map_err(|e| e.to_string())?;

    if !resp.status().is_success() {
        return Err(format!("HTTP {}", resp.status()));
    }

    let release: serde_json::Value = resp.json().await.map_err(|e| e.to_string())?;
    let version = release.get("tag_name")
        .and_then(|v| v.as_str())
        .unwrap_or("unknown")
        .to_string();

    Ok(version)
}

// ==================== 数据结构 ====================

/// 本地端口信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LocalPort {
    pub protocol: String,
    pub ip: String,
    pub port: u16,
}

// ==================== 应用初始化 ====================

pub fn init_app(app: &mut tauri::App) {
    let config_dir = app.path().app_config_dir()
        .unwrap_or_else(|_| PathBuf::from("."))
        .join("frpc-gui");

    let config_path = config_dir.join("frpc.json");
    let config_manager = ConfigManager::new(config_path);

    let settings_path = config_dir.join("settings.json");
    let settings_manager = SettingsManager::new(settings_path);

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
