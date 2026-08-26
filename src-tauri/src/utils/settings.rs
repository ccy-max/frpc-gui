//! 应用设置持久化模块
//!
//! 高可用设计：
//! - 原子写入（临时文件 + rename），崩溃/断电不会损坏 settings.json
//! - 版本字段 + 迁移逻辑，向后兼容旧版本数据
//!
//! 注：frpc 路径/配置文件/日志目录已移除——多进程架构下
//! frpc 由「版本管理」统一管理（激活版本机制），
//! 配置与日志按服务器自动生成于 servers/{id}/ 目录。

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

/// 当前设置数据版本
const SETTINGS_VERSION: u32 = 2;

/// 应用设置（持久化到 JSON 文件）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppSettings {
    /// 数据版本号（迁移用）
    #[serde(default = "default_version")]
    pub version: u32,
    pub language: String,
    pub theme: String,
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
        Self {
            version: SETTINGS_VERSION,
            language: "zh-CN".to_string(),
            theme: "auto".to_string(),
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

    /// 加载设置（带版本迁移）
    pub fn load(&self) -> Result<AppSettings> {
        if !self.settings_path.exists() {
            return Ok(AppSettings::default());
        }
        let content = fs::read_to_string(&self.settings_path)
            .with_context(|| "读取设置文件失败")?;

        let mut value: serde_json::Value =
            serde_json::from_str(&content).with_context(|| "解析设置文件失败")?;
        let version = value.get("version").and_then(|v| v.as_u64()).unwrap_or(0);

        // v0/v1 → v2：v1 含 frpc_path/config_path/log_path（已废弃，直接剥离）
        if version < SETTINGS_VERSION as u64 {
            if let Some(obj) = value.as_object_mut() {
                obj.remove("frpc_path");
                obj.remove("config_path");
                obj.remove("log_path");
            }
            value["version"] = serde_json::json!(SETTINGS_VERSION);
            log::info!("Migrated settings v{} → v{} (removed legacy path fields)", version, SETTINGS_VERSION);
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

        let temp_path = self.settings_path.with_extension("json.tmp");
        fs::write(&temp_path, &content)
            .with_context(|| format!("写入临时文件失败: {:?}", temp_path))?;
        fs::rename(&temp_path, &self.settings_path)
            .with_context(|| "原子替换设置文件失败")?;
        if temp_path.exists() {
            let _ = fs::remove_file(&temp_path);
        }
        Ok(())
    }
}
