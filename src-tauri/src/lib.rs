use serde::{Deserialize, Serialize};
use tauri::Manager;

/// 应用配置结构
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppConfig {
    pub language: String,
    pub theme: String,
    pub frp_binary_path: String,
    pub config_path: String,
    pub log_path: String,
    pub auto_start: bool,
    pub minimize_to_tray: bool,
    pub close_to_tray: bool,
    pub check_update_on_start: bool,
    pub download_mirror: Option<String>,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            language: "zh-CN".to_string(),
            theme: "auto".to_string(),
            frp_binary_path: String::new(),
            config_path: String::new(),
            log_path: String::new(),
            auto_start: false,
            minimize_to_tray: true,
            close_to_tray: true,
            check_update_on_start: true,
            download_mirror: None,
        }
    }
}

/// FRP 服务器配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerConfig {
    pub id: String,
    pub name: String,
    pub server_addr: String,
    pub server_port: u16,
    pub auth_method: Option<String>,
    pub token: Option<String>,
    pub user: Option<String>,
    pub meta_token: Option<String>,
    pub tls_enable: Option<bool>,
    pub log_level: Option<String>,
    pub log_max_days: Option<u32>,
    pub admin_addr: Option<String>,
    pub admin_port: Option<u16>,
    pub admin_user: Option<String>,
    pub admin_password: Option<String>,
    pub enabled: bool,
    pub created_at: u64,
    pub updated_at: u64,
}

/// FRP 代理配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProxyConfig {
    pub name: String,
    #[serde(rename = "type")]
    pub proxy_type: String,
    pub local_ip: Option<String>,
    pub local_port: Option<u16>,
    pub remote_port: Option<u16>,
    pub custom_domains: Option<Vec<String>>,
    pub subdomain: Option<String>,
    pub locations: Option<Vec<String>>,
    pub http_user: Option<String>,
    pub http_password: Option<String>,
    pub use_encryption: Option<bool>,
    pub use_compression: Option<bool>,
    pub secret_key: Option<String>,
    pub role: Option<String>,
    pub server_name: Option<String>,
    pub enabled: bool,
    pub created_at: u64,
    pub updated_at: u64,
}

/// Tauri 命令定义
#[tauri::command]
fn greet(name: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust!", name)
}

#[tauri::command]
fn load_config(app_handle: tauri::AppHandle) -> Result<AppConfig, String> {
    // TODO: 从文件加载配置
    Ok(app_handle
        .state::<Option<AppConfig>>()
        .inner()
        .clone()
        .unwrap_or_default())
}

#[tauri::command]
fn save_config(app_handle: tauri::AppHandle, config: AppConfig) -> Result<(), String> {
    // TODO: 保存配置到文件
    log::info!("Saving config: {:?}", config);
    Ok(())
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    // 初始化日志
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info")).init();

    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_fs::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_notification::init())
        .plugin(tauri_plugin_autostart::init(
            tauri_plugin_autostart::MacosLauncher::LaunchAgent,
            Some(vec!["--flag1", "--flag2"]),
        ))
        .manage::<Option<AppConfig>>(None)
        .invoke_handler(tauri::generate_handler![
            greet,
            load_config,
            save_config
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
