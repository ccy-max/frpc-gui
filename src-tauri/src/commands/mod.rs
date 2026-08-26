//! Tauri 命令处理模块

use crate::frp::{
    ConfigManager, FrpConfig, FrpProcessManager, FrpVersionInfo, FrpVersionManager, MirrorInfo,
    ProcessState, validate_config,
};
use crate::utils::settings::{AppSettings, SettingsManager};
use log::{error, info, warn};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use tauri::{Manager, State};
use tokio::sync::{mpsc, Mutex};

// ==================== 应用状态 ====================

use std::collections::HashMap;

pub struct AppState {
    // 多个进程管理器，key 为服务器 ID
    pub process_managers: Mutex<HashMap<String, FrpProcessManager>>,
    // 多个配置管理器，key 为服务器 ID
    pub config_managers: Mutex<HashMap<String, ConfigManager>>,
    pub log_tx: Mutex<Option<mpsc::Sender<String>>>,
    pub settings_manager: Mutex<Option<SettingsManager>>,
    pub version_manager: Mutex<Option<FrpVersionManager>>,
}

impl AppState {
    pub fn new() -> Self {
        let (log_tx, _) = mpsc::channel(100);
        Self {
            process_managers: Mutex::new(HashMap::new()),
            config_managers: Mutex::new(HashMap::new()),
            log_tx: Mutex::new(Some(log_tx)),
            settings_manager: Mutex::new(None),
            version_manager: Mutex::new(None),
        }
    }

    pub fn with_config(config_manager: ConfigManager) -> Self {
        let (log_tx, _) = mpsc::channel(100);
        Self {
            process_managers: Mutex::new(HashMap::new()),
            config_managers: Mutex::new(HashMap::new()),
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

// ==================== 进程控制（旧版单进程 API - 已废弃） ====================
// 注意：以下 API 已被新的多进程 API 替代
// 请使用 start_server, stop_server, restart_server 等新 API

#[derive(Debug, Serialize, Deserialize)]
pub struct ProcessStatusResponse {
    pub running: bool,
    pub pid: Option<u32>,
    pub state: String,
    pub connection_error: Option<String>,
    pub last_start_time: Option<i64>,
}

#[deprecated(note = "请使用 start_server 代替")]
#[tauri::command]
pub async fn start_frp(config: FrpConfig, state: State<'_, AppState>) -> Result<bool, String> {
    // 临时实现：启动默认服务器
    let server_id = "default".to_string();
    start_server(server_id, config, state).await
}

#[deprecated(note = "请使用 stop_server 代替")]
#[tauri::command]
pub async fn stop_frp(state: State<'_, AppState>) -> Result<bool, String> {
    stop_server("default".to_string(), state).await
}

#[deprecated(note = "请使用 restart_server 代替")]
#[tauri::command]
pub async fn restart_frp(config: FrpConfig, state: State<'_, AppState>) -> Result<bool, String> {
    restart_server("default".to_string(), config, state).await
}

#[deprecated(note = "请使用 get_server_status 代替")]
#[tauri::command]
pub async fn get_process_status(state: State<'_, AppState>) -> Result<ProcessStatusResponse, String> {
    let status = get_server_status("default".to_string(), state).await?;
    Ok(ProcessStatusResponse {
        running: status.running,
        pid: status.pid,
        state: status.state,
        connection_error: status.error,
        last_start_time: None,
    })
}

#[deprecated(note = "请使用新的多进程 API")]
#[tauri::command]
pub async fn detect_frpc_process(state: State<'_, AppState>) -> Result<bool, String> {
    // 简化实现
    Ok(false)
}

#[deprecated(note = "请使用新的多进程 API")]
#[tauri::command]
pub async fn reload_frp(config: FrpConfig, state: State<'_, AppState>) -> Result<bool, String> {
    restart_server("default".to_string(), config, state).await
}

#[deprecated(note = "请使用新的多进程 API")]
#[tauri::command]
pub async fn modify_proxy_status(
    proxy_name: String,
    enabled: bool,
    state: State<'_, AppState>,
) -> Result<bool, String> {
    // 简化实现：找到第一个包含该代理的服务器
    let servers_guard = state.config_managers.lock().await;
    // 这里需要更复杂的逻辑，暂时返回成功
    drop(servers_guard);
    Ok(true)
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

    // 1. 停止所有 FRP 进程
    let mut pm_guard = state.process_managers.lock().await;
    for (_, pm) in pm_guard.iter_mut() {
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
    let mut cm_guard = state.config_managers.lock().await;
    cm_guard.clear();
    drop(cm_guard);

    let mut pm_guard = state.process_managers.lock().await;
    pm_guard.clear();
    drop(pm_guard);

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

// ==================== 服务器和代理持久化 ====================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PersistentData {
    pub servers: Vec<serde_json::Value>,
    pub proxies: Vec<serde_json::Value>,
}

impl Default for PersistentData {
    fn default() -> Self {
        Self {
            servers: Vec::new(),
            proxies: Vec::new(),
        }
    }
}

fn get_persistent_data_path() -> Result<PathBuf, String> {
    let config_dir = dirs::config_dir()
        .map(|d| d.join("frpc-gui"))
        .unwrap_or_else(|| PathBuf::from("."));
    Ok(config_dir.join("frpc-gui-data.json"))
}

/// 加载持久化的服务器和代理数据
#[tauri::command]
pub fn load_persistent_data() -> Result<PersistentData, String> {
    let path = get_persistent_data_path()?;
    if !path.exists() {
        return Ok(PersistentData::default());
    }
    let content = std::fs::read_to_string(&path)
        .map_err(|e| format!("读取持久化数据失败：{}", e))?;
    serde_json::from_str(&content)
        .map_err(|e| format!("解析持久化数据失败：{}", e))
}

/// 保存持久化的服务器和代理数据
#[tauri::command]
pub fn save_persistent_data(data: PersistentData) -> Result<bool, String> {
    let path = get_persistent_data_path()?;
    let content = serde_json::to_string_pretty(&data)
        .map_err(|e| format!("序列化持久化数据失败：{}", e))?;
    std::fs::write(&path, content)
        .map_err(|e| format!("保存持久化数据失败：{}", e))?;
    info!("Persistent data saved to {:?}", path);
    Ok(true)
}

// ==================== 多 FRP 进程支持 ====================

use reqwest::Client;
use once_cell::sync::Lazy;

// 监控数据互斥锁（防止并发写入）
static MONITORING_DATA_MUTEX: Lazy<Mutex<()>> = Lazy::new(|| Mutex::new(()));

/// 查询 FRP Admin API
async fn query_admin_api<T: serde::de::DeserializeOwned>(
    addr: &str,
    port: u16,
    user: &str,
    password: &str,
    path: &str,
) -> Result<T, String> {
    let client = Client::builder()
        .timeout(std::time::Duration::from_secs(5))
        .build()
        .map_err(|e| format!("创建 HTTP 客户端失败：{}", e))?;

    let url = format!("http://{}:{}{}", addr, port, path);
    
    let response = client
        .get(&url)
        .basic_auth(user, Some(password))
        .send()
        .await
        .map_err(|e| format!("请求 Admin API 失败：{}", e))?;

    if !response.status().is_success() {
        return Err(format!("Admin API 返回错误状态：{}", response.status()));
    }

    let data: T = response
        .json()
        .await
        .map_err(|e| format!("解析 Admin API 响应失败：{}", e))?;

    Ok(data)
}

/// 启动指定服务器的 FRP 进程
#[tauri::command]
pub async fn start_server(
    server_id: String,
    config: FrpConfig,
    state: State<'_, AppState>,
) -> Result<bool, String> {
    info!("Starting FRP for server: {}", server_id);

    let mut pm_guard = state.process_managers.lock().await;
    
    // 检查是否已在运行
    if pm_guard.contains_key(&server_id) {
        return Err(format!("服务器 {} 的 FRP 进程已在运行中", server_id));
    }

    // 获取配置目录
    let server_config_dir = get_server_config_dir(&server_id)?;
    std::fs::create_dir_all(&server_config_dir)
        .map_err(|e| format!("创建配置目录失败：{}", e))?;

    let config_path = server_config_dir.join("config.toml");
    let log_file_path = server_config_dir.join("frpc.log");

    // 保存配置
    let cm = ConfigManager::new(config_path.clone());
    let toml_content = cm.generate_toml(&config, log_file_path.to_string_lossy().as_ref())
        .map_err(|e| format!("生成配置失败：{}", e))?;
    std::fs::write(&config_path, toml_content)
        .map_err(|e| format!("写入配置文件失败：{}", e))?;

    // 获取 frpc 路径
    #[cfg(windows)]
    let default_frpc = "frpc.exe";
    #[cfg(not(windows))]
    let default_frpc = "frpc";

    let vm_guard = state.version_manager.lock().await;
    let frpc_path = vm_guard.as_ref()
        .and_then(|vm| vm.get_downloaded_frpc_path())
        .map(|p| p.to_string_lossy().to_string())
        .unwrap_or_else(|| default_frpc.to_string());
    drop(vm_guard);

    // 获取日志通道
    let log_tx = {
        let tx_guard = state.log_tx.lock().await;
        match tx_guard.clone() {
            Some(tx) => tx,
            None => return Err("日志系统未初始化".to_string()),
        }
    };

    // 创建进程管理器
    let mut pm = FrpProcessManager::new(
        PathBuf::from(&frpc_path),
        config_path,
        log_tx,
    );

    // 启动进程
    match pm.start(&config).await {
        Ok(_) => {
            let pid = pm.get_pid();
            pm_guard.insert(server_id.clone(), pm);
            info!("FRP started for server {}: PID={}", server_id, pid);
            Ok(true)
        }
        Err(e) => {
            error!("Failed to start FRP for server {}: {}", server_id, e);
            Err(e.to_string())
        }
    }
}

/// 停止指定服务器的 FRP 进程
#[tauri::command]
pub async fn stop_server(
    server_id: String,
    state: State<'_, AppState>,
) -> Result<bool, String> {
    info!("Stopping FRP for server: {}", server_id);

    let mut pm_guard = state.process_managers.lock().await;
    
    match pm_guard.remove(&server_id) {
        Some(mut pm) => {
            match pm.stop().await {
                Ok(_) => {
                    info!("FRP stopped for server {}", server_id);
                    Ok(true)
                }
                Err(e) => {
                    error!("Failed to stop FRP for server {}: {}", server_id, e);
                    Err(e.to_string())
                }
            }
        }
        None => {
            info!("FRP process not found for server {}", server_id);
            Ok(true) // 进程不存在也算成功
        }
    }
}

/// 重启指定服务器的 FRP 进程
#[tauri::command]
pub async fn restart_server(
    server_id: String,
    config: FrpConfig,
    state: State<'_, AppState>,
) -> Result<bool, String> {
    info!("Restarting FRP for server: {}", server_id);

    let mut pm_guard = state.process_managers.lock().await;
    
    match pm_guard.get_mut(&server_id) {
        Some(pm) => {
            match pm.restart(&config).await {
                Ok(_) => {
                    info!("FRP restarted for server {}", server_id);
                    Ok(true)
                }
                Err(e) => {
                    error!("Failed to restart FRP for server {}: {}", server_id, e);
                    Err(e.to_string())
                }
            }
        }
        None => {
            // 如果进程不存在，先启动
            drop(pm_guard);
            start_server(server_id, config, state).await
        }
    }
}

/// 获取服务器状态
#[tauri::command]
pub async fn get_server_status(
    server_id: String,
    state: State<'_, AppState>,
) -> Result<ServerStatusResponse, String> {
    let pm_guard = state.process_managers.lock().await;
    
    match pm_guard.get(&server_id) {
        Some(pm) => {
            let proc_state = pm.get_state();
            let (running, pid, state_str) = match proc_state {
                ProcessState::Running { pid } => (true, Some(pid), "running".to_string()),
                ProcessState::Starting => (false, None, "starting".to_string()),
                ProcessState::Stopping => (false, None, "stopping".to_string()),
                ProcessState::Stopped => (false, None, "stopped".to_string()),
                ProcessState::Error(e) => (false, None, format!("error: {}", e)),
            };
            
            Ok(ServerStatusResponse {
                server_id,
                server_name: String::new(), // 前端会填充
                running,
                pid,
                state: state_str,
                proxy_count: 0, // 前端会填充
                error: None,
            })
        }
        None => {
            Ok(ServerStatusResponse {
                server_id,
                server_name: String::new(),
                running: false,
                pid: None,
                state: "not_started".to_string(),
                proxy_count: 0,
                error: None,
            })
        }
    }
}

/// 获取所有服务器状态
#[tauri::command]
pub async fn get_all_servers_status(
    state: State<'_, AppState>,
) -> Result<Vec<ServerStatusResponse>, String> {
    let pm_guard = state.process_managers.lock().await;
    let mut statuses = Vec::new();
    
    for (server_id, pm) in pm_guard.iter() {
        let proc_state = pm.get_state();
        let (running, pid, state_str) = match proc_state {
            ProcessState::Running { pid } => (true, Some(pid), "running".to_string()),
            ProcessState::Starting => (false, None, "starting".to_string()),
            ProcessState::Stopping => (false, None, "stopping".to_string()),
            ProcessState::Stopped => (false, None, "stopped".to_string()),
            ProcessState::Error(e) => (false, None, format!("error: {}", e)),
        };
        
        statuses.push(ServerStatusResponse {
            server_id: server_id.clone(),
            server_name: String::new(),
            running,
            pid,
            state: state_str,
            proxy_count: 0,
            error: None,
        });
    }
    
    Ok(statuses)
}

/// 获取所有代理的状态和流量信息
#[tauri::command]
pub async fn get_all_proxy_status(
    state: State<'_, AppState>,
) -> Result<Vec<ProxyStatusInfo>, String> {
    let pm_guard = state.process_managers.lock().await;
    let mut proxy_statuses = Vec::new();
    
    for (server_id, pm) in pm_guard.iter() {
        // 获取进程状态
        let proc_state = pm.get_state();
        let is_running = matches!(proc_state, ProcessState::Running { .. });
        
        if !is_running {
            continue;
        }
        
        // 这里需要从配置中获取 admin 配置
        // 简化处理：假设 admin 配置为默认值
        let admin_addr = "127.0.0.1";
        let admin_port = 7400; // 需要从配置中读取
        let admin_user = "admin";
        let admin_password = "admin";
        
        // 查询 Admin API
        match query_admin_api::<AdminStatusResponse>(
            admin_addr,
            admin_port,
            admin_user,
            admin_password,
            "/api/status",
        ).await {
            Ok(status) => {
                for proxy in status.proxies {
                    proxy_statuses.push(ProxyStatusInfo {
                        name: proxy.proxy_name.clone(),
                        server_id: server_id.clone(),
                        state: proxy.status.clone(),
                        err_msg: proxy.err.clone(),
                        today_traffic_in: proxy.today_traffic_in,
                        today_traffic_out: proxy.today_traffic_out,
                        last_start_time: proxy.last_start_time,
                        last_close_time: proxy.last_close_time,
                    });
                }
            }
            Err(e) => {
                // Admin API 不可用时，使用进程状态作为代理状态
                proxy_statuses.push(ProxyStatusInfo {
                    name: format!("{}-proxy", server_id),
                    server_id: server_id.clone(),
                    state: if is_running { "online".to_string() } else { "offline".to_string() },
                    err_msg: None,
                    today_traffic_in: 0,
                    today_traffic_out: 0,
                    last_start_time: None,
                    last_close_time: None,
                });
            }
        }
    }
    
    Ok(proxy_statuses)
}

/// 获取指定服务器的流量统计
#[tauri::command]
pub async fn get_server_traffic(
    server_id: String,
    state: State<'_, AppState>,
) -> Result<TrafficStatistics, String> {
    let pm_guard = state.process_managers.lock().await;
    
    match pm_guard.get(&server_id) {
        Some(pm) => {
            let proc_state = pm.get_state();
            let is_running = matches!(proc_state, ProcessState::Running { .. });
            
            if !is_running {
                return Ok(TrafficStatistics {
                    server_id,
                    total_traffic_in: 0,
                    total_traffic_out: 0,
                    today_traffic_in: 0,
                    today_traffic_out: 0,
                    proxies: Vec::new(),
                });
            }
            
            // 查询 Admin API 获取流量
            let admin_addr = "127.0.0.1";
            let admin_port = 7400;
            let admin_user = "admin";
            let admin_password = "admin";
            
            match query_admin_api::<AdminStatusResponse>(
                admin_addr,
                admin_port,
                admin_user,
                admin_password,
                "/api/status",
            ).await {
                Ok(status) => {
                    let mut total_in = 0u64;
                    let mut total_out = 0u64;
                    let mut proxy_traffic_list = Vec::new();
                    
                    for proxy in status.proxies {
                        total_in += proxy.today_traffic_in;
                        total_out += proxy.today_traffic_out;
                        
                        proxy_traffic_list.push(ProxyTrafficInfo {
                            name: proxy.proxy_name,
                            traffic_in: proxy.today_traffic_in,
                            traffic_out: proxy.today_traffic_out,
                            today_traffic_in: proxy.today_traffic_in,
                            today_traffic_out: proxy.today_traffic_out,
                        });
                    }
                    
                    // 持久化今日流量（使用互斥锁防止并发）
                    let _guard = MONITORING_DATA_MUTEX.lock().unwrap();
                    
                    let today = get_today_date();
                    let mut monitoring_data = load_monitoring_data()?;
                    
                    // 查找或创建今日记录
                    let today_record = monitoring_data.traffic_history
                        .iter_mut()
                        .find(|r| r.date == today);
                    
                    if let Some(record) = today_record {
                        // 更新今日记录
                        record.total_traffic_in = total_in;
                        record.total_traffic_out = total_out;
                        record.proxies = proxy_traffic_list.iter().map(|p| ProxyDailyTraffic {
                            name: p.name.clone(),
                            traffic_in: p.today_traffic_in,
                            traffic_out: p.today_traffic_out,
                        }).collect();
                    } else {
                        // 创建新记录
                        monitoring_data.traffic_history.push(TrafficHistory {
                            date: today,
                            total_traffic_in: total_in,
                            total_traffic_out: total_out,
                            proxies: proxy_traffic_list.iter().map(|p| ProxyDailyTraffic {
                                name: p.name.clone(),
                                traffic_in: p.today_traffic_in,
                                traffic_out: p.today_traffic_out,
                            }).collect(),
                        });
                        // 只保留最近 30 天
                        if monitoring_data.traffic_history.len() > 30 {
                            monitoring_data.traffic_history.remove(0);
                        }
                    }
                    
                    monitoring_data.last_updated = chrono::Utc::now().timestamp();
                    save_monitoring_data(&monitoring_data)?;
                    
                    // 锁在这里自动释放
                    
                    Ok(TrafficStatistics {
                        server_id,
                        total_traffic_in: total_in,
                        total_traffic_out: total_out,
                        today_traffic_in: total_in,
                        today_traffic_out: total_out,
                        proxies: proxy_traffic_list,
                    })
                }
                Err(_) => {
                    // Admin API 不可用，返回空数据
                    Ok(TrafficStatistics {
                        server_id,
                        total_traffic_in: 0,
                        total_traffic_out: 0,
                        today_traffic_in: 0,
                        today_traffic_out: 0,
                        proxies: Vec::new(),
                    })
                }
            }
        }
        None => {
            Err(format!("服务器 {} 未找到", server_id))
        }
    }
}

/// 获取流量历史记录
#[tauri::command]
pub async fn get_traffic_history(
    days: Option<u32>,
) -> Result<Vec<TrafficHistory>, String> {
    let monitoring_data = load_monitoring_data()?;
    let days = days.unwrap_or(30) as usize;
    
    let history: Vec<TrafficHistory> = monitoring_data.traffic_history
        .into_iter()
        .rev()
        .take(days)
        .collect();
    
    Ok(history)
}

/// 获取连接状态历史
#[tauri::command]
pub async fn get_connection_history(
    proxy_name: Option<String>,
    server_id: Option<String>,
) -> Result<Vec<ConnectionHistory>, String> {
    let monitoring_data = load_monitoring_data()?;
    
    let history = if proxy_name.is_some() || server_id.is_some() {
        monitoring_data.connection_history
            .into_iter()
            .filter(|h| {
                (proxy_name.is_none() || h.proxy_name == proxy_name.as_ref().unwrap()) &&
                (server_id.is_none() || h.server_id == server_id.as_ref().unwrap())
            })
            .collect()
    } else {
        monitoring_data.connection_history
    };
    
    Ok(history)
}

/// 记录连接事件
#[tauri::command]
pub async fn log_connection_event(
    proxy_name: String,
    server_id: String,
    event_type: String,
    message: Option<String>,
    duration_secs: Option<u64>,
) -> Result<(), String> {
    // 使用互斥锁防止并发写入
    let _guard = MONITORING_DATA_MUTEX.lock().unwrap();
    
    let mut monitoring_data = load_monitoring_data()?;
    
    // 查找或创建代理的历史记录
    let history = monitoring_data.connection_history
        .iter_mut()
        .find(|h| h.proxy_name == proxy_name && h.server_id == server_id);
    
    let event = ConnectionEvent {
        timestamp: chrono::Utc::now().timestamp(),
        event_type,
        message,
        duration_secs,
    };
    
    if let Some(hist) = history {
        hist.events.push(event);
        // 只保留最近 100 个事件
        if hist.events.len() > 100 {
            hist.events.remove(0);
        }
    } else {
        monitoring_data.connection_history.push(ConnectionHistory {
            proxy_name,
            server_id,
            events: vec![event],
        });
    }
    
    monitoring_data.last_updated = chrono::Utc::now().timestamp();
    save_monitoring_data(&monitoring_data)?;
    
    // 锁在这里自动释放
    Ok(())
}

// ==================== 数据结构 ====================

/// 本地端口信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LocalPort {
    pub protocol: String,
    pub ip: String,
    pub port: u16,
}

/// 服务器状态响应
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerStatusResponse {
    pub server_id: String,
    pub server_name: String,
    pub running: bool,
    pub pid: Option<u32>,
    pub state: String,
    pub proxy_count: usize,
    pub error: Option<String>,
}

/// 代理状态信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProxyStatusInfo {
    pub name: String,
    pub server_id: String,
    pub state: String,  // online, offline, starting
    pub err_msg: Option<String>,
    pub today_traffic_in: u64,  // 今日下载流量 (bytes)
    pub today_traffic_out: u64, // 今日上传流量 (bytes)
    pub last_start_time: Option<i64>,
    pub last_close_time: Option<i64>,
}

/// 流量统计信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrafficStatistics {
    pub server_id: String,
    pub total_traffic_in: u64,   // 总下载 (bytes)
    pub total_traffic_out: u64,  // 总上传 (bytes)
    pub today_traffic_in: u64,   // 今日下载
    pub today_traffic_out: u64,  // 今日上传
    pub proxies: Vec<ProxyTrafficInfo>,
}

/// 单个代理的流量信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProxyTrafficInfo {
    pub name: String,
    pub traffic_in: u64,
    pub traffic_out: u64,
    pub today_traffic_in: u64,
    pub today_traffic_out: u64,
}

/// 流量历史记录（按日期）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrafficHistory {
    pub date: String,  // YYYY-MM-DD
    pub total_traffic_in: u64,
    pub total_traffic_out: u64,
    pub proxies: Vec<ProxyDailyTraffic>,
}

/// 代理每日流量
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProxyDailyTraffic {
    pub name: String,
    pub traffic_in: u64,
    pub traffic_out: u64,
}

/// 连接状态历史
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConnectionHistory {
    pub proxy_name: String,
    pub server_id: String,
    pub events: Vec<ConnectionEvent>,
}

/// 连接事件
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConnectionEvent {
    pub timestamp: i64,
    pub event_type: String,  // connected, disconnected, error
    pub message: Option<String>,
    pub duration_secs: Option<u64>,
}

/// 持久化监控数据
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct MonitoringData {
    pub version: u32,  // 数据版本号，用于迁移
    pub traffic_history: Vec<TrafficHistory>,  // 最近 30 天
    pub connection_history: Vec<ConnectionHistory>,
    pub last_updated: i64,
}

impl MonitoringData {
    pub fn new() -> Self {
        Self {
            version: 1,
            traffic_history: Vec::new(),
            connection_history: Vec::new(),
            last_updated: 0,
        }
    }
}

/// Admin API 响应结构
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AdminStatusResponse {
    pub proxies: Vec<AdminProxyStatus>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AdminProxyStatus {
    #[serde(rename = "name")]
    pub proxy_name: String,
    pub conf_name: String,
    #[serde(rename = "type")]
    pub proxy_type: String,
    pub status: String,  // "online" or "offline"
    pub err: Option<String>,
    pub today_traffic_in: u64,
    pub today_traffic_out: u64,
    pub cur_conns: u32,
    pub last_start_time: Option<i64>,
    pub last_close_time: Option<i64>,
}

// ==================== 工具函数 ====================

/// 获取服务器的配置目录
fn get_server_config_dir(server_id: &str) -> Result<PathBuf, String> {
    let config_dir = dirs::config_dir()
        .map(|d| d.join("frpc-gui"))
        .unwrap_or_else(|| PathBuf::from("."));
    Ok(config_dir.join("servers").join(format!("server-{}", server_id)))
}

/// 获取监控数据文件路径
fn get_monitoring_data_path() -> Result<PathBuf, String> {
    let config_dir = dirs::config_dir()
        .map(|d| d.join("frpc-gui"))
        .unwrap_or_else(|| PathBuf::from("."));
    Ok(config_dir.join("monitoring-data.json"))
}

/// 加载监控数据（带版本迁移）
fn load_monitoring_data() -> Result<MonitoringData, String> {
    let path = get_monitoring_data_path()?;
    if !path.exists() {
        return Ok(MonitoringData::new());
    }
    
    let content = std::fs::read_to_string(&path)
        .map_err(|e| format!("读取监控数据失败：{}", e))?;
    
    // 先解析为 Value 以检查版本
    let mut value: serde_json::Value = serde_json::from_str(&content)
        .map_err(|e| format!("解析监控数据失败：{}", e))?;
    
    // 获取版本号（默认为 0，表示旧版本）
    let version = value.get("version")
        .and_then(|v| v.as_u64())
        .unwrap_or(0);
    
    // 版本迁移
    match version {
        0 => {
            // v0 -> v1: 添加 version 字段
            value["version"] = serde_json::json!(1);
            info!("Migrating monitoring data from v0 to v1");
        }
        1 => {
            // 当前版本，无需迁移
        }
        v => {
            // 未来版本，警告但尝试加载
            warn!("Unknown monitoring data version: {}, attempting to load", v);
        }
    }
    
    // 反序列化为结构体
    serde_json::from_value(value)
        .map_err(|e| format!("反序列化监控数据失败：{}", e))
}

/// 保存监控数据（原子写入）
fn save_monitoring_data(data: &MonitoringData) -> Result<(), String> {
    let path = get_monitoring_data_path()?;
    
    // 确保父目录存在
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)
            .map_err(|e| format!("创建目录失败：{}", e))?;
    }
    
    // 序列化为 JSON
    let content = serde_json::to_string_pretty(data)
        .map_err(|e| format!("序列化监控数据失败：{}", e))?;
    
    // 写入临时文件（原子操作第一步）
    let temp_path = path.with_extension("json.tmp");
    std::fs::write(&temp_path, &content)
        .map_err(|e| format!("写入临时文件失败：{}", e))?;
    
    // 原子重命名（确保要么完整写入，要么不变）
    std::fs::rename(&temp_path, &path)
        .map_err(|e| format!("重命名文件失败：{}", e))?;
    
    // 清理可能存在的旧临时文件（忽略错误）
    if temp_path.exists() {
        let _ = std::fs::remove_file(&temp_path);
    }
    
    info!("Monitoring data saved atomically to {:?}", path);
    Ok(())
}

/// 获取今日日期字符串（使用本地时区）
fn get_today_date() -> String {
    // 使用 chrono 的本地时区，确保跨时区环境正确
    chrono::Local::now().format("%Y-%m-%d").to_string()
}

// ==================== 应用初始化 ====================

pub fn init_app(app: &mut tauri::App) {
    let config_dir = app.path().app_config_dir()
        .unwrap_or_else(|_| PathBuf::from("."))
        .join("frpc-gui");

    let settings_path = config_dir.join("settings.json");
    let settings_manager = SettingsManager::new(settings_path);

    let install_dir = config_dir.join("bin");
    std::fs::create_dir_all(&install_dir).ok();
    let version_manager = FrpVersionManager::new(install_dir);

    let (log_tx, _) = mpsc::channel(100);
    let app_state = AppState {
        process_managers: Mutex::new(HashMap::new()),
        config_managers: Mutex::new(HashMap::new()),
        log_tx: Mutex::new(Some(log_tx)),
        settings_manager: Mutex::new(Some(settings_manager)),
        version_manager: Mutex::new(Some(version_manager)),
    };
    app.manage(app_state);

    info!("Application initialized with multi-process support");
}
