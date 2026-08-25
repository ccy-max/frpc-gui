//! 应用设置持久化模块

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

/// 应用设置（持久化到 JSON 文件）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppSettings {
    pub language: String,
    pub theme: String,
    pub frpc_path: String,
    pub config_path: String,
    pub log_path: String,
    pub auto_start: bool,
    pub minimize_to_tray: bool,
    pub close_to_tray: bool,
    pub default_server_id: Option<String>,  // 默认服务器 ID
}

impl Default for AppSettings {
    fn default() -> Self {
        let config_dir = dirs::config_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join("frpc-gui");
        Self {
            language: "zh-CN".to_string(),
            theme: "auto".to_string(),
            frpc_path: String::new(),
            config_path: config_dir.join("frpc.toml").to_string_lossy().to_string(),
            log_path: config_dir.join("logs").to_string_lossy().to_string(),
            auto_start: false,
            minimize_to_tray: true,
            close_to_tray: true,
            default_server_id: None,
        }
    }
}

/// 设置管理器
pub struct SettingsManager {
    settings_path: PathBuf,
}

impl SettingsManager {
    pub fn new(settings_path: PathBuf) -> Self {
        Self { settings_path }
    }

    /// 加载设置
    pub fn load(&self) -> Result<AppSettings> {
        if !self.settings_path.exists() {
            return Ok(AppSettings::default());
        }
        let content = fs::read_to_string(&self.settings_path)
            .with_context(|| "读取设置文件失败")?;
        let settings: AppSettings = serde_json::from_str(&content)
            .with_context(|| "解析设置文件失败")?;
        Ok(settings)
    }

    /// 保存设置
    pub fn save(&self, settings: &AppSettings) -> Result<()> {
        if let Some(parent) = self.settings_path.parent() {
            fs::create_dir_all(parent)?;
        }
        let content = serde_json::to_string_pretty(settings)?;
        fs::write(&self.settings_path, content)?;
        Ok(())
    }
}
