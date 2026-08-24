//! FRP 配置文件管理模块

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};

/// FRP 完整配置结构
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FrpConfig {
    pub server_addr: String,
    pub server_port: u16,
    pub auth: AuthConfig,
    pub user: Option<String>,
    pub tls: TlsConfig,
    pub log: LogConfig,
    pub admin: AdminConfig,
    pub proxies: Vec<ProxyConfig>,
}

impl Default for FrpConfig {
    fn default() -> Self {
        Self {
            server_addr: "127.0.0.1".to_string(),
            server_port: 7000,
            auth: AuthConfig::default(),
            user: None,
            tls: TlsConfig::default(),
            log: LogConfig::default(),
            admin: AdminConfig::default(),
            proxies: Vec::new(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthConfig {
    pub method: String,
    pub token: Option<String>,
    pub additional: Option<String>,
}

impl Default for AuthConfig {
    fn default() -> Self {
        Self {
            method: "token".to_string(),
            token: None,
            additional: None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TlsConfig {
    pub enable: bool,
    pub cert_file: Option<String>,
    pub key_file: Option<String>,
    pub trusted_ca_file: Option<String>,
}

impl Default for TlsConfig {
    fn default() -> Self {
        Self {
            enable: false,
            cert_file: None,
            key_file: None,
            trusted_ca_file: None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogConfig {
    pub to: String,
    pub level: String,
    pub max_days: u32,
}

impl Default for LogConfig {
    fn default() -> Self {
        Self {
            to: "console".to_string(),
            level: "info".to_string(),
            max_days: 3,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AdminConfig {
    pub addr: String,
    pub port: u16,
    pub user: Option<String>,
    pub password: Option<String>,
}

impl Default for AdminConfig {
    fn default() -> Self {
        Self {
            addr: "127.0.0.1".to_string(),
            port: 7400,
            user: Some("admin".to_string()),
            password: Some("admin".to_string()),
        }
    }
}

/// 代理配置
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
}

impl Default for ProxyConfig {
    fn default() -> Self {
        Self {
            name: String::new(),
            proxy_type: "tcp".to_string(),
            local_ip: Some("127.0.0.1".to_string()),
            local_port: Some(8080),
            remote_port: Some(8080),
            custom_domains: None,
            subdomain: None,
            locations: None,
            http_user: None,
            http_password: None,
            use_encryption: None,
            use_compression: None,
            secret_key: None,
            role: None,
            server_name: None,
            enabled: true,
        }
    }
}

/// 配置文件管理器
pub struct ConfigManager {
    config_path: PathBuf,
}

impl ConfigManager {
    pub fn new(config_path: PathBuf) -> Self {
        Self { config_path }
    }

    /// 加载配置
    pub fn load(&self) -> Result<FrpConfig> {
        if !self.config_path.exists() {
            return Ok(FrpConfig::default());
        }

        let content = fs::read_to_string(&self.config_path)
            .with_context(|| format!("Failed to read config file: {:?}", self.config_path))?;

        // 尝试 TOML 格式
        if self.config_path.extension().map_or(false, |ext| ext == "toml") {
            let config: FrpConfig = toml::from_str(&content)
                .with_context(|| "Failed to parse TOML config")?;
            Ok(config)
        } else {
            // JSON 格式
            let config: FrpConfig = serde_json::from_str(&content)
                .with_context(|| "Failed to parse JSON config")?;
            Ok(config)
        }
    }

    /// 保存配置
    pub fn save(&self, config: &FrpConfig) -> Result<()> {
        // 确保目录存在
        if let Some(parent) = self.config_path.parent() {
            fs::create_dir_all(parent)
                .with_context(|| format!("Failed to create directory: {:?}", parent))?;
        }

        let content = if self.config_path.extension().map_or(false, |ext| ext == "toml") {
            toml::to_string_pretty(config)
                .with_context(|| "Failed to serialize config to TOML")?
        } else {
            serde_json::to_string_pretty(config)
                .with_context(|| "Failed to serialize config to JSON")?
        };

        fs::write(&self.config_path, content)
            .with_context(|| format!("Failed to write config file: {:?}", self.config_path))?;

        Ok(())
    }

    /// 导出配置到文件
    pub fn export_to(&self, target_path: &Path) -> Result<()> {
        let config = self.load()?;
        let content = serde_json::to_string_pretty(&config)?;
        fs::write(target_path, content)?;
        Ok(())
    }

    /// 从文件导入配置
    pub fn import_from(&self, source_path: &Path) -> Result<()> {
        let content = fs::read_to_string(source_path)?;
        let config: FrpConfig = serde_json::from_str(&content)?;
        self.save(&config)?;
        Ok(())
    }
}

/// 验证配置
pub fn validate_config(config: &FrpConfig) -> Result<(), String> {
    if config.server_addr.is_empty() {
        return Err("服务器地址不能为空".to_string());
    }

    if config.server_port == 0 {
        return Err("服务器端口无效".to_string());
    }

    for proxy in &config.proxies {
        if proxy.name.is_empty() {
            return Err(format!("代理名称不能为空"));
        }

        if proxy.proxy_type.is_empty() {
            return Err(format!("代理 {} 的类型不能为空", proxy.name));
        }

        // TCP/UDP 需要 local_port 和 remote_port
        if ["tcp", "udp"].contains(&proxy.proxy_type.as_str()) {
            if proxy.local_port.is_none() || proxy.remote_port.is_none() {
                return Err(format!("代理 {} 需要配置本地端口和远程端口", proxy.name));
            }
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = FrpConfig::default();
        assert_eq!(config.server_port, 7000);
        assert_eq!(config.auth.method, "token");
    }
}
