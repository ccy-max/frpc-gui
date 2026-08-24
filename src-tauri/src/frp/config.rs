//! FRP 配置文件管理模块
//!
//! 负责将内部配置结构序列化为 frpc 可识别的 TOML 格式，
//! 以及从 TOML/JSON 文件加载配置。

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};

// ==================== 配置结构定义 ====================

/// FRP 完整配置结构（内部表示，包含 UI 状态字段）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FrpConfig {
    pub server_addr: String,
    pub server_port: u16,
    #[serde(default)]
    pub user: Option<String>,
    #[serde(default)]
    pub auth: AuthConfig,
    #[serde(default)]
    pub tls: TlsConfig,
    #[serde(default)]
    pub log: LogConfig,
    #[serde(default)]
    pub admin: AdminConfig,
    #[serde(default)]
    pub transport: TransportConfig,
    #[serde(default)]
    pub web_server: WebServerConfig,
    #[serde(default)]
    pub metadatas: Option<BTreeMap<String, String>>,
    #[serde(default)]
    pub login_fail_exit: Option<bool>,
    #[serde(default)]
    pub udp_packet_size: Option<u32>,
    #[serde(default)]
    pub proxies: Vec<ProxyConfig>,
    #[serde(default)]
    pub visitors: Vec<ProxyConfig>,
}

impl Default for FrpConfig {
    fn default() -> Self {
        Self {
            server_addr: "127.0.0.1".to_string(),
            server_port: 7000,
            user: None,
            auth: AuthConfig::default(),
            tls: TlsConfig::default(),
            log: LogConfig::default(),
            admin: AdminConfig::default(),
            transport: TransportConfig::default(),
            web_server: WebServerConfig::default(),
            metadatas: None,
            login_fail_exit: Some(false),
            udp_packet_size: Some(1500),
            proxies: Vec::new(),
            visitors: Vec::new(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct AuthConfig {
    pub method: String,
    #[serde(default)]
    pub token: Option<String>,
    #[serde(default)]
    pub additional: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TlsConfig {
    pub enable: bool,
    #[serde(default)]
    pub cert_file: Option<String>,
    #[serde(default)]
    pub key_file: Option<String>,
    #[serde(default)]
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
    #[serde(default)]
    pub user: Option<String>,
    #[serde(default)]
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

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct TransportConfig {
    #[serde(default)]
    pub dial_server_timeout: Option<u32>,
    #[serde(default)]
    pub dial_server_keepalive: Option<u32>,
    #[serde(default)]
    pub pool_count: Option<u32>,
    #[serde(default)]
    pub tcp_mux: Option<bool>,
    #[serde(default)]
    pub tcp_mux_keepalive_interval: Option<u32>,
    #[serde(default)]
    pub protocol: Option<String>,
    #[serde(default)]
    pub connect_server_local_ip: Option<String>,
    #[serde(default)]
    pub proxy_url: Option<String>,
    #[serde(default)]
    pub heartbeat_interval: Option<u32>,
    #[serde(default)]
    pub heartbeat_timeout: Option<u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WebServerConfig {
    pub addr: String,
    pub port: u16,
    #[serde(default)]
    pub user: Option<String>,
    #[serde(default)]
    pub password: Option<String>,
}

impl Default for WebServerConfig {
    fn default() -> Self {
        Self {
            addr: "127.0.0.1".to_string(),
            port: 57400,
            user: None,
            password: None,
        }
    }
}

/// 代理配置（内部表示，包含 UI 状态字段如 enabled）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProxyConfig {
    pub name: String,
    #[serde(rename = "type")]
    pub proxy_type: String,
    #[serde(default)]
    pub local_ip: Option<String>,
    #[serde(default)]
    pub local_port: Option<String>,
    #[serde(default)]
    pub remote_port: Option<String>,
    #[serde(default)]
    pub custom_domains: Option<Vec<String>>,
    #[serde(default)]
    pub subdomain: Option<String>,
    #[serde(default)]
    pub locations: Option<Vec<String>>,
    #[serde(default)]
    pub host_header_rewrite: Option<String>,
    #[serde(default)]
    pub http_user: Option<String>,
    #[serde(default)]
    pub http_password: Option<String>,
    #[serde(default)]
    pub use_encryption: Option<bool>,
    #[serde(default)]
    pub use_compression: Option<bool>,
    #[serde(default)]
    pub proxy_protocol_version: Option<String>,
    #[serde(default)]
    pub secret_key: Option<String>,
    #[serde(default)]
    pub role: Option<String>,
    #[serde(default)]
    pub server_name: Option<String>,
    #[serde(default)]
    pub server_user: Option<String>,
    #[serde(default)]
    pub bind_addr: Option<String>,
    #[serde(default)]
    pub bind_port: Option<u16>,
    #[serde(default)]
    pub keep_tunnel_open: Option<bool>,
    #[serde(default)]
    pub fallback_to: Option<String>,
    #[serde(default)]
    pub fallback_timeout_ms: Option<u32>,
    #[serde(default)]
    pub https2http: Option<bool>,
    #[serde(default)]
    pub https2http_ca_file: Option<String>,
    #[serde(default)]
    pub https2http_key_file: Option<String>,
    /// UI 状态字段，不写入 frpc.toml
    #[serde(default)]
    pub enabled: bool,
    /// 批量端口标记（如 "8080-8090" 或 "8080,8081"）
    #[serde(default)]
    pub is_range_port: Option<bool>,
    /// visitors 标记
    #[serde(default)]
    pub visitors_model: Option<String>,
}

impl Default for ProxyConfig {
    fn default() -> Self {
        Self {
            name: String::new(),
            proxy_type: "tcp".to_string(),
            local_ip: Some("127.0.0.1".to_string()),
            local_port: Some("8080".to_string()),
            remote_port: Some("8080".to_string()),
            custom_domains: None,
            subdomain: None,
            locations: None,
            host_header_rewrite: None,
            http_user: None,
            http_password: None,
            use_encryption: None,
            use_compression: None,
            proxy_protocol_version: None,
            secret_key: None,
            role: None,
            server_name: None,
            server_user: None,
            bind_addr: None,
            bind_port: None,
            keep_tunnel_open: None,
            fallback_to: None,
            fallback_timeout_ms: None,
            https2http: None,
            https2http_ca_file: None,
            https2http_key_file: None,
            enabled: true,
            is_range_port: None,
            visitors_model: None,
        }
    }
}

// ==================== 配置文件管理器 ====================

pub struct ConfigManager {
    config_path: PathBuf,
}

impl ConfigManager {
    pub fn new(config_path: PathBuf) -> Self {
        Self { config_path }
    }

    /// 加载配置（JSON 格式，包含 UI 状态字段）
    pub fn load(&self) -> Result<FrpConfig> {
        if !self.config_path.exists() {
            return Ok(FrpConfig::default());
        }
        let content = fs::read_to_string(&self.config_path)
            .with_context(|| format!("读取配置文件失败: {:?}", self.config_path))?;

        // 统一用 JSON 加载（内部配置包含 enabled 等 UI 字段）
        let config: FrpConfig = serde_json::from_str(&content)
            .with_context(|| "解析 JSON 配置失败")?;
        Ok(config)
    }

    /// 保存配置（JSON 格式，包含 UI 状态字段）
    pub fn save(&self, config: &FrpConfig) -> Result<()> {
        if let Some(parent) = self.config_path.parent() {
            fs::create_dir_all(parent)
                .with_context(|| format!("创建目录失败: {:?}", parent))?;
        }
        let content = serde_json::to_string_pretty(config)?;
        fs::write(&self.config_path, content)?;
        Ok(())
    }

    /// 生成 frpc 可识别的 TOML 配置文件
    ///
    /// 关键逻辑：
    /// 1. 排除 UI 状态字段（enabled, is_range_port, visitors_model）
    /// 2. 按代理类型分组处理
    /// 3. 处理批量端口（模板渲染）
    /// 4. 处理 visitors
    /// 5. 处理 https2http 插件
    pub fn generate_toml(&self, config: &FrpConfig, log_file_path: &str) -> Result<String> {
        let mut toml = String::new();

        // 1. 基本配置
        toml.push_str(&format!("serverAddr = \"{}\"\n", config.server_addr));
        toml.push_str(&format!("serverPort = {}\n", config.server_port));

        if let Some(ref user) = config.user {
            if !user.is_empty() {
                toml.push_str(&format!("user = \"{}\"\n", user));
            }
        }

        // 2. 认证配置
        if !config.auth.method.is_empty() && config.auth.method != "none" {
            toml.push_str("\n[auth]\n");
            toml.push_str(&format!("method = \"{}\"\n", config.auth.method));
            if let Some(ref token) = config.auth.token {
                if !token.is_empty() {
                    toml.push_str(&format!("token = \"{}\"\n", token));
                }
            }
        }

        // 3. 日志配置
        toml.push_str("\n[log]\n");
        toml.push_str(&format!("to = \"{}\"\n", log_file_path));
        toml.push_str(&format!("level = \"{}\"\n", config.log.level));
        toml.push_str(&format!("maxDays = {}\n", config.log.max_days));

        // 4. 传输配置
        let t = &config.transport;
        toml.push_str("\n[transport]\n");
        if let Some(v) = t.dial_server_timeout { toml.push_str(&format!("dialServerTimeout = {}\n", v)); }
        if let Some(v) = t.dial_server_keepalive { toml.push_str(&format!("dialServerKeepalive = {}\n", v)); }
        if let Some(v) = t.pool_count { toml.push_str(&format!("poolCount = {}\n", v)); }
        if let Some(v) = t.tcp_mux { toml.push_str(&format!("tcpMux = {}\n", v)); }
        if let Some(v) = t.tcp_mux_keepalive_interval { toml.push_str(&format!("tcpMuxKeepaliveInterval = {}\n", v)); }
        if let Some(ref v) = t.protocol { toml.push_str(&format!("protocol = \"{}\"\n", v)); }
        if let Some(ref v) = t.heartbeat_interval { toml.push_str(&format!("heartbeatInterval = {}\n", v)); }
        if let Some(ref v) = t.heartbeat_timeout { toml.push_str(&format!("heartbeatTimeout = {}\n", v)); }

        // 5. TLS 配置
        if config.tls.enable {
            toml.push_str("\n[transport.tls]\n");
            toml.push_str("enable = true\n");
            if let Some(ref v) = config.tls.cert_file { toml.push_str(&format!("certFile = \"{}\"\n", v)); }
            if let Some(ref v) = config.tls.key_file { toml.push_str(&format!("keyFile = \"{}\"\n", v)); }
            if let Some(ref v) = config.tls.trusted_ca_file { toml.push_str(&format!("trustedCaFile = \"{}\"\n", v)); }
        }

        // 6. Web Server 配置
        toml.push_str("\n[webServer]\n");
        toml.push_str(&format!("addr = \"{}\"\n", config.web_server.addr));
        toml.push_str(&format!("port = {}\n", config.web_server.port));
        if let Some(ref v) = config.web_server.user {
            if !v.is_empty() { toml.push_str(&format!("user = \"{}\"\n", v)); }
        }
        if let Some(ref v) = config.web_server.password {
            if !v.is_empty() { toml.push_str(&format!("password = \"{}\"\n", v)); }
        }

        // 7. Metadatas
        if let Some(ref metadatas) = config.metadatas {
            if !metadatas.is_empty() {
                toml.push_str("\n[metadatas]\n");
                for (k, v) in metadatas {
                    toml.push_str(&format!("{} = \"{}\"\n", k, v));
                }
            }
        }

        // 8. loginFailExit
        if let Some(v) = config.login_fail_exit {
            toml.push_str(&format!("\nloginFailExit = {}\n", v));
        }

        // 9. udpPacketSize
        if let Some(v) = config.udp_packet_size {
            toml.push_str(&format!("udpPacketSize = {}\n", v));
        }

        // 10. 启用的普通代理（排除批量端口和 visitors）
        let enabled_proxies: Vec<&ProxyConfig> = config.proxies.iter()
            .filter(|p| p.enabled)
            .filter(|p| !Self::is_range_port(p))
            .filter(|p| !Self::is_visitor(p))
            .collect();

        for proxy in enabled_proxies {
            toml.push_str(&Self::generate_proxy_toml(proxy));
        }

        // 11. 批量端口代理（展开为多个代理）
        for proxy in config.proxies.iter().filter(|p| p.enabled && Self::is_range_port(p)) {
            toml.push_str(&Self::generate_range_port_toml(proxy));
        }

        // 12. Visitors
        for visitor in config.visitors.iter().filter(|v| v.enabled) {
            toml.push_str(&Self::generate_visitor_toml(visitor));
        }

        Ok(toml)
    }

    /// 判断是否为批量端口代理
    fn is_range_port(proxy: &ProxyConfig) -> bool {
        if !["tcp", "udp"].contains(&proxy.proxy_type.as_str()) {
            return false;
        }
        if let Some(ref lp) = proxy.local_port {
            return lp.contains('-') || lp.contains(',');
        }
        false
    }

    /// 判断是否为 visitor
    fn is_visitor(proxy: &ProxyConfig) -> bool {
        proxy.visitors_model.as_deref() == Some("visitors")
            && ["stcp", "sudp", "xtcp"].contains(&proxy.proxy_type.as_str())
    }

    /// 生成单个代理的 TOML
    fn generate_proxy_toml(proxy: &ProxyConfig) -> String {
        let mut s = String::new();
        s.push_str(&format!("\n[[proxies]]\n"));
        s.push_str(&format!("name = \"{}\"\n", proxy.name));
        s.push_str(&format!("type = \"{}\"\n", proxy.proxy_type));

        match proxy.proxy_type.as_str() {
            "tcp" | "udp" => {
                if let Some(ref v) = proxy.local_ip { s.push_str(&format!("localIP = \"{}\"\n", v)); }
                if let Some(ref v) = proxy.local_port {
                    if let Ok(port) = v.parse::<u16>() { s.push_str(&format!("localPort = {}\n", port)); }
                }
                if let Some(ref v) = proxy.remote_port {
                    if let Ok(port) = v.parse::<u16>() { s.push_str(&format!("remotePort = {}\n", port)); }
                }
            }
            "http" | "https" => {
                if let Some(ref v) = proxy.local_ip { s.push_str(&format!("localIP = \"{}\"\n", v)); }
                if let Some(ref v) = proxy.local_port {
                    if let Ok(port) = v.parse::<u16>() { s.push_str(&format!("localPort = {}\n", port)); }
                }
                if let Some(ref v) = proxy.custom_domains {
                    if !v.is_empty() { s.push_str(&format!("customDomains = {}\n", Self::format_array(v))); }
                }
                if let Some(ref v) = proxy.subdomain {
                    if !v.is_empty() { s.push_str(&format!("subdomain = \"{}\"\n", v)); }
                }
                if let Some(ref v) = proxy.locations {
                    if !v.is_empty() { s.push_str(&format!("locations = {}\n", Self::format_array(v))); }
                }
                if let Some(ref v) = proxy.host_header_rewrite {
                    if !v.is_empty() { s.push_str(&format!("hostHeaderRewrite = \"{}\"\n", v)); }
                }
                if let Some(ref v) = proxy.http_user {
                    if !v.is_empty() { s.push_str(&format!("httpUser = \"{}\"\n", v)); }
                }
                if let Some(ref v) = proxy.http_password {
                    if !v.is_empty() { s.push_str(&format!("httpPassword = \"{}\"\n", v)); }
                }
                // https2http 插件
                if proxy.https2http.unwrap_or(false) && proxy.proxy_type == "https" {
                    s.push_str(&format!("[proxies.plugin]\n"));
                    s.push_str("type = \"https2http\"\n");
                    if let Some(ref ip) = proxy.local_ip {
                        if let Some(ref port) = proxy.local_port {
                            s.push_str(&format!("localAddr = \"{}:{}\"\n", ip, port));
                        }
                    }
                    if let Some(ref v) = proxy.https2http_ca_file {
                        if !v.is_empty() { s.push_str(&format!("crtPath = \"{}\"\n", v)); }
                    }
                    if let Some(ref v) = proxy.https2http_key_file {
                        if !v.is_empty() { s.push_str(&format!("keyPath = \"{}\"\n", v)); }
                    }
                }
            }
            "stcp" | "xtcp" | "sudp" => {
                if let Some(ref v) = proxy.local_ip { s.push_str(&format!("localIP = \"{}\"\n", v)); }
                if let Some(ref v) = proxy.local_port {
                    if let Ok(port) = v.parse::<u16>() { s.push_str(&format!("localPort = {}\n", port)); }
                }
                if let Some(ref v) = proxy.secret_key {
                    if !v.is_empty() { s.push_str(&format!("secretKey = \"{}\"\n", v)); }
                }
            }
            _ => {}
        }

        // transport 子配置
        if proxy.use_encryption.unwrap_or(false) || proxy.use_compression.unwrap_or(false) {
            s.push_str("\n[proxies.transport]\n");
            if proxy.use_encryption.unwrap_or(false) { s.push_str("useEncryption = true\n"); }
            if proxy.use_compression.unwrap_or(false) { s.push_str("useCompression = true\n"); }
            if let Some(ref v) = proxy.proxy_protocol_version {
                if !v.is_empty() { s.push_str(&format!("proxyProtocolVersion = \"{}\"\n", v)); }
            }
        }

        s
    }

    /// 生成批量端口代理的 TOML（展开端口范围）
    fn generate_range_port_toml(proxy: &ProxyConfig) -> String {
        let local_ports = Self::parse_port_range(proxy.local_port.as_deref().unwrap_or(""));
        let remote_ports = Self::parse_port_range(proxy.remote_port.as_deref().unwrap_or(""));

        let mut s = String::new();
        for (i, (lp, rp)) in local_ports.iter().zip(remote_ports.iter()).enumerate() {
            s.push_str(&format!("\n[[proxies]]\n"));
            s.push_str(&format!("name = \"{}-{}\"\n", proxy.name, i + 1));
            s.push_str(&format!("type = \"{}\"\n", proxy.proxy_type));
            if let Some(ref v) = proxy.local_ip { s.push_str(&format!("localIP = \"{}\"\n", v)); }
            s.push_str(&format!("localPort = {}\n", lp));
            s.push_str(&format!("remotePort = {}\n", rp));
        }
        s
    }

    /// 生成 visitor 的 TOML
    fn generate_visitor_toml(visitor: &ProxyConfig) -> String {
        let mut s = String::new();
        s.push_str(&format!("\n[[visitors]]\n"));
        s.push_str(&format!("name = \"{}\"\n", visitor.name));
        s.push_str(&format!("type = \"{}\"\n", visitor.proxy_type));
        if let Some(ref v) = visitor.server_name { s.push_str(&format!("serverName = \"{}\"\n", v)); }
        if let Some(ref v) = visitor.secret_key { s.push_str(&format!("secretKey = \"{}\"\n", v)); }
        if let Some(ref v) = visitor.bind_addr { s.push_str(&format!("bindAddr = \"{}\"\n", v)); }
        if let Some(v) = visitor.bind_port { s.push_str(&format!("bindPort = {}\n", v)); }
        if let Some(ref v) = visitor.server_user {
            if !v.is_empty() { s.push_str(&format!("serverUser = \"{}\"\n", v)); }
        }
        if visitor.proxy_type == "xtcp" {
            if let Some(v) = visitor.keep_tunnel_open { s.push_str(&format!("keepTunnelOpen = {}\n", v)); }
            if let Some(ref v) = visitor.fallback_to {
                if !v.is_empty() { s.push_str(&format!("fallbackTo = \"{}\"\n", v)); }
            }
            if let Some(v) = visitor.fallback_timeout_ms { s.push_str(&format!("fallbackTimeoutMs = {}\n", v)); }
        }
        s
    }

    /// 解析端口范围字符串（如 "8080-8083" 或 "8080,8081"）
    fn parse_port_range(s: &str) -> Vec<u16> {
        if s.contains('-') {
            let parts: Vec<&str> = s.split('-').collect();
            if parts.len() == 2 {
                if let (Ok(start), Ok(end)) = (parts[0].parse::<u16>(), parts[1].parse::<u16>()) {
                    return (start..=end).collect();
                }
            }
        } else if s.contains(',') {
            return s.split(',')
                .filter_map(|p| p.trim().parse::<u16>().ok())
                .collect();
        }
        // 单个端口
        if let Ok(port) = s.parse::<u16>() {
            return vec![port];
        }
        vec![]
    }

    /// 格式化数组为 TOML 格式
    fn format_array(arr: &[String]) -> String {
        let items: Vec<String> = arr.iter().map(|s| format!("\"{}\"", s)).collect();
        format!("[{}]", items.join(", "))
    }

    /// 导出配置到文件（JSON 格式，包含 UI 状态）
    pub fn export_to(&self, target_path: &Path) -> Result<()> {
        let config = self.load()?;
        let content = serde_json::to_string_pretty(&config)?;
        fs::write(target_path, content)?;
        Ok(())
    }

    /// 从文件导入配置（支持 JSON 格式）
    pub fn import_from(&self, source_path: &Path) -> Result<()> {
        let content = fs::read_to_string(source_path)?;
        let config: FrpConfig = serde_json::from_str(&content)?;
        self.save(&config)?;
        Ok(())
    }

    /// 从 frpc.toml 导入配置（解析标准 frp TOML 格式）
    pub fn import_toml(&self, toml_path: &Path) -> Result<FrpConfig> {
        let content = fs::read_to_string(toml_path)?;
        let value: toml::Value = toml::from_str(&content).with_context(|| "解析 TOML 失败")?;

        let mut config = FrpConfig::default();

        // 解析基本字段
        if let Some(v) = value.get("serverAddr").and_then(|v| v.as_str()) {
            config.server_addr = v.to_string();
        }
        if let Some(v) = value.get("serverPort").and_then(|v| v.as_integer()) {
            config.server_port = v as u16;
        }
        if let Some(v) = value.get("user").and_then(|v| v.as_str()) {
            config.user = Some(v.to_string());
        }

        // 解析 auth
        if let Some(auth) = value.get("auth").and_then(|v| v.as_table()) {
            if let Some(m) = auth.get("method").and_then(|v| v.as_str()) {
                config.auth.method = m.to_string();
            }
            if let Some(t) = auth.get("token").and_then(|v| v.as_str()) {
                config.auth.token = Some(t.to_string());
            }
        }

        // 解析 log
        if let Some(log) = value.get("log").and_then(|v| v.as_table()) {
            if let Some(l) = log.get("level").and_then(|v| v.as_str()) {
                config.log.level = l.to_string();
            }
            if let Some(d) = log.get("maxDays").and_then(|v| v.as_integer()) {
                config.log.max_days = d as u32;
            }
        }

        // 解析 transport
        if let Some(t) = value.get("transport").and_then(|v| v.as_table()) {
            if let Some(v) = t.get("protocol").and_then(|v| v.as_str()) {
                config.transport.protocol = Some(v.to_string());
            }
            if let Some(v) = t.get("poolCount").and_then(|v| v.as_integer()) {
                config.transport.pool_count = Some(v as u32);
            }
            if let Some(v) = t.get("tcpMux").and_then(|v| v.as_bool()) {
                config.transport.tcp_mux = Some(v);
            }
            if let Some(v) = t.get("heartbeatInterval").and_then(|v| v.as_integer()) {
                config.transport.heartbeat_interval = Some(v as u32);
            }
            if let Some(v) = t.get("heartbeatTimeout").and_then(|v| v.as_integer()) {
                config.transport.heartbeat_timeout = Some(v as u32);
            }
        }

        // 解析 TLS
        if let Some(tls) = value.get("transport").and_then(|v| v.get("tls")).and_then(|v| v.as_table()) {
            if let Some(e) = tls.get("enable").and_then(|v| v.as_bool()) {
                config.tls.enable = e;
            }
            if let Some(v) = tls.get("certFile").and_then(|v| v.as_str()) {
                config.tls.cert_file = Some(v.to_string());
            }
            if let Some(v) = tls.get("keyFile").and_then(|v| v.as_str()) {
                config.tls.key_file = Some(v.to_string());
            }
            if let Some(v) = tls.get("trustedCaFile").and_then(|v| v.as_str()) {
                config.tls.trusted_ca_file = Some(v.to_string());
            }
        }

        // 解析 webServer
        if let Some(ws) = value.get("webServer").and_then(|v| v.as_table()) {
            if let Some(v) = ws.get("addr").and_then(|v| v.as_str()) {
                config.web_server.addr = v.to_string();
            }
            if let Some(v) = ws.get("port").and_then(|v| v.as_integer()) {
                config.web_server.port = v as u16;
            }
            if let Some(v) = ws.get("user").and_then(|v| v.as_str()) {
                config.web_server.user = Some(v.to_string());
            }
            if let Some(v) = ws.get("password").and_then(|v| v.as_str()) {
                config.web_server.password = Some(v.to_string());
            }
        }

        // 解析 metadatas
        if let Some(meta) = value.get("metadatas").and_then(|v| v.as_table()) {
            let mut m = BTreeMap::new();
            for (k, v) in meta {
                if let Some(s) = v.as_str() {
                    m.insert(k.clone(), s.to_string());
                }
            }
            if !m.is_empty() { config.metadatas = Some(m); }
        }

        // 解析 proxies
        if let Some(proxies) = value.get("proxies").and_then(|v| v.as_array()) {
            for p in proxies {
                let mut proxy = ProxyConfig::default();
                if let Some(v) = p.get("name").and_then(|v| v.as_str()) { proxy.name = v.to_string(); }
                if let Some(v) = p.get("type").and_then(|v| v.as_str()) { proxy.proxy_type = v.to_string(); }
                if let Some(v) = p.get("localIP").and_then(|v| v.as_str()) { proxy.local_ip = Some(v.to_string()); }
                if let Some(v) = p.get("localPort").and_then(|v| v.as_integer()) {
                    proxy.local_port = Some(v.to_string());
                }
                if let Some(v) = p.get("remotePort").and_then(|v| v.as_integer()) {
                    proxy.remote_port = Some(v.to_string());
                }
                if let Some(v) = p.get("subdomain").and_then(|v| v.as_str()) { proxy.subdomain = Some(v.to_string()); }
                if let Some(v) = p.get("secretKey").and_then(|v| v.as_str()) { proxy.secret_key = Some(v.to_string()); }
                if let Some(v) = p.get("customDomains").and_then(|v| v.as_array()) {
                    proxy.custom_domains = Some(v.iter().filter_map(|x| x.as_str().map(String::from)).collect());
                }
                if let Some(v) = p.get("locations").and_then(|v| v.as_array()) {
                    proxy.locations = Some(v.iter().filter_map(|x| x.as_str().map(String::from)).collect());
                }
                if let Some(v) = p.get("httpUser").and_then(|v| v.as_str()) { proxy.http_user = Some(v.to_string()); }
                if let Some(v) = p.get("httpPassword").and_then(|v| v.as_str()) { proxy.http_password = Some(v.to_string()); }

                // transport 子配置
                if let Some(t) = p.get("transport").and_then(|v| v.as_table()) {
                    if let Some(v) = t.get("useEncryption").and_then(|v| v.as_bool()) { proxy.use_encryption = Some(v); }
                    if let Some(v) = t.get("useCompression").and_then(|v| v.as_bool()) { proxy.use_compression = Some(v); }
                    if let Some(v) = t.get("proxyProtocolVersion").and_then(|v| v.as_str()) { proxy.proxy_protocol_version = Some(v.to_string()); }
                }

                proxy.enabled = true;
                config.proxies.push(proxy);
            }
        }

        // 解析 visitors
        if let Some(visitors) = value.get("visitors").and_then(|v| v.as_array()) {
            for v in visitors {
                let mut visitor = ProxyConfig::default();
                if let Some(v) = v.get("name").and_then(|v| v.as_str()) { visitor.name = v.to_string(); }
                if let Some(v) = v.get("type").and_then(|v| v.as_str()) { visitor.proxy_type = v.to_string(); }
                if let Some(v) = v.get("serverName").and_then(|v| v.as_str()) { visitor.server_name = Some(v.to_string()); }
                if let Some(v) = v.get("secretKey").and_then(|v| v.as_str()) { visitor.secret_key = Some(v.to_string()); }
                if let Some(v) = v.get("bindAddr").and_then(|v| v.as_str()) { visitor.bind_addr = Some(v.to_string()); }
                if let Some(v) = v.get("bindPort").and_then(|v| v.as_integer()) { visitor.bind_port = Some(v as u16); }
                if let Some(v) = v.get("serverUser").and_then(|v| v.as_str()) { visitor.server_user = Some(v.to_string()); }
                if let Some(v) = v.get("keepTunnelOpen").and_then(|v| v.as_bool()) { visitor.keep_tunnel_open = Some(v); }
                if let Some(v) = v.get("fallbackTo").and_then(|v| v.as_str()) { visitor.fallback_to = Some(v.to_string()); }
                if let Some(v) = v.get("fallbackTimeoutMs").and_then(|v| v.as_integer()) { visitor.fallback_timeout_ms = Some(v as u32); }
                visitor.enabled = true;
                visitor.visitors_model = Some("visitors".to_string());
                config.visitors.push(visitor);
            }
        }

        // 解析 loginFailExit
        if let Some(v) = value.get("loginFailExit").and_then(|v| v.as_bool()) {
            config.login_fail_exit = Some(v);
        }

        // 保存并返回
        self.save(&config)?;
        Ok(config)
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
            return Err("代理名称不能为空".to_string());
        }
        if proxy.proxy_type.is_empty() {
            return Err(format!("代理 {} 的类型不能为空", proxy.name));
        }
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

    #[test]
    fn test_parse_port_range_dash() {
        let ports = ConfigManager::parse_port_range("8080-8083");
        assert_eq!(ports, vec![8080, 8081, 8082, 8083]);
    }

    #[test]
    fn test_parse_port_range_comma() {
        let ports = ConfigManager::parse_port_range("8080,8081,8082");
        assert_eq!(ports, vec![8080, 8081, 8082]);
    }

    #[test]
    fn test_parse_port_single() {
        let ports = ConfigManager::parse_port_range("8080");
        assert_eq!(ports, vec![8080]);
    }

    #[test]
    fn test_generate_proxy_toml_tcp() {
        let proxy = ProxyConfig {
            name: "test".to_string(),
            proxy_type: "tcp".to_string(),
            local_ip: Some("127.0.0.1".to_string()),
            local_port: Some("8080".to_string()),
            remote_port: Some("8080".to_string()),
            enabled: true,
            ..Default::default()
        };
        let toml = ConfigManager::generate_proxy_toml(&proxy);
        assert!(toml.contains("name = \"test\""));
        assert!(toml.contains("type = \"tcp\""));
        assert!(toml.contains("localPort = 8080"));
        assert!(toml.contains("remotePort = 8080"));
        // enabled 字段不应出现在 TOML 中
        assert!(!toml.contains("enabled"));
    }

    #[test]
    fn test_generate_toml_excludes_ui_fields() {
        let config = FrpConfig {
            server_addr: "127.0.0.1".to_string(),
            server_port: 7000,
            proxies: vec![ProxyConfig {
                name: "test".to_string(),
                proxy_type: "tcp".to_string(),
                local_ip: Some("127.0.0.1".to_string()),
                local_port: Some("8080".to_string()),
                remote_port: Some("8080".to_string()),
                enabled: true,
                is_range_port: Some(false),
                visitors_model: None,
                ..Default::default()
            }],
            ..Default::default()
        };
        let toml = config_manager_test_generate(&config);
        assert!(!toml.contains("enabled"));
        assert!(!toml.contains("is_range_port"));
        assert!(!toml.contains("visitors_model"));
    }

    fn config_manager_test_generate(config: &FrpConfig) -> String {
        let cm = ConfigManager::new(PathBuf::from("/tmp/test_config.json"));
        cm.generate_toml(config, "/tmp/frpc.log").unwrap_or_default()
    }
}
