//! 应用设置持久化模块
//!
//! 高可用设计：
//! - 原子写入（临时文件 + rename），崩溃/断电不会损坏 settings.json
//! - 版本字段 + 迁移逻辑，向后兼容旧版本数据

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

/// 当前设置数据版本
const SETTINGS_VERSION: u32 = 1;

/// 应用设置（持久化到 JSON 文件）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppSettings {
    /// 数据版本号（迁移用）
    #[serde(default = "default_version")]
    pub version: u32,
    pub language: String,
    pub theme: String,
    pub frpc_path: String,
    pub config_path: String,
    pub log_path: String,
    pub auto_start: bool,
    pub minimize_to_tray: bool,
    pub close_to_tray: bool,
    #[serde(default)]
    pub default_server_id: Option<String>, // 默认服务器 ID
}

fn default_version() -> u32 {
    SETTINGS_VERSION
}

impl Default for AppSettings {
    fn default() -> Self {
        let config_dir = dirs::config_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join("frpc-gui");
        Self {
            version: SETTINGS_VERSION,
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

    /// 加载设置（带版本迁移：v0 → v1 补齐 version 字段）
    pub fn load(&self) -> Result<AppSettings> {
        if !self.settings_path.exists() {
            return Ok(AppSettings::default());
        }
        let content = fs::read_to_string(&self.settings_path)
            .with_context(|| "读取设置文件失败")?;

        // 先解析为 Value 做版本检查与迁移
        let mut value: serde_json::Value =
            serde_json::from_str(&content).with_context(|| "解析设置文件失败")?;
        let version = value.get("version").and_then(|v| v.as_u64()).unwrap_or(0);
        if version < SETTINGS_VERSION as u64 {
            value["version"] = serde_json::json!(SETTINGS_VERSION);
            log::info!("Migrating settings from v{} to v{}", version, SETTINGS_VERSION);
        }

        let settings: AppSettings =
            serde_json::from_value(value).with_context(|| "反序列化设置失败")?;
        Ok(settings)
    }

    /// 保存设置（原子写入：临时文件 + rename，杜绝半写损坏）
    pub fn save(&self, settings: &AppSettings) -> Result<()> {
        if let Some(parent) = self.settings_path.parent() {
            fs::create_dir_all(parent)?;
        }
        let content = serde_json::to_string_pretty(settings)?;

        // 1. 写入同目录临时文件
        let temp_path = self.settings_path.with_extension("json.tmp");
        fs::write(&temp_path, &content)
            .with_context(|| format!("写入临时文件失败: {:?}", temp_path))?;

        // 2. 原子重命名覆盖目标（POSIX/Windows 均保证原子性）
        fs::rename(&temp_path, &self.settings_path)
            .with_context(|| "原子替换设置文件失败")?;

        // 3. 清理残留临时文件（rename 成功后不应存在，防御性清理）
        if temp_path.exists() {
            let _ = fs::remove_file(&temp_path);
        }
        Ok(())
    }
}
