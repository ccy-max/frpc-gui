//! 工具函数模块

use anyhow::Result;
use std::path::PathBuf;

/// 获取应用数据目录
pub fn get_app_data_dir() -> Result<PathBuf> {
    let dir = dirs::data_local_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("frpc-gui");
    
    std::fs::create_dir_all(&dir)?;
    Ok(dir)
}

/// 获取配置目录
pub fn get_config_dir() -> Result<PathBuf> {
    let dir = dirs::config_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("frpc-gui");
    
    std::fs::create_dir_all(&dir)?;
    Ok(dir)
}

/// 获取日志目录
pub fn get_log_dir() -> Result<PathBuf> {
    let dir = get_app_data_dir()?
        .join("logs");
    
    std::fs::create_dir_all(&dir)?;
    Ok(dir)
}

/// 格式化文件大小
pub fn format_file_size(bytes: u64) -> String {
    const KB: u64 = 1024;
    const MB: u64 = KB * 1024;
    const GB: u64 = MB * 1024;

    if bytes >= GB {
        format!("{:.2} GB", bytes as f64 / GB as f64)
    } else if bytes >= MB {
        format!("{:.2} MB", bytes as f64 / MB as f64)
    } else if bytes >= KB {
        format!("{:.2} KB", bytes as f64 / KB as f64)
    } else {
        format!("{} B", bytes)
    }
}

/// 格式化持续时间
pub fn format_duration(seconds: u64) -> String {
    let hours = seconds / 3600;
    let minutes = (seconds % 3600) / 60;
    let secs = seconds % 60;

    if hours > 0 {
        format!("{}h {}m {}s", hours, minutes, secs)
    } else if minutes > 0 {
        format!("{}m {}s", minutes, secs)
    } else {
        format!("{}s", secs)
    }
}

/// 检查端口是否可用
#[cfg(unix)]
pub fn is_port_available(port: u16) -> bool {
    use std::net::TcpListener;
    TcpListener::bind(format!("127.0.0.1:{}", port)).is_ok()
}

#[cfg(windows)]
pub fn is_port_available(port: u16) -> bool {
    use std::net::TcpListener;
    TcpListener::bind(format!("127.0.0.1:{}", port)).is_ok()
}

/// 获取平台信息
pub fn get_platform_info() -> (String, String) {
    let os = std::env::consts::OS;
    let arch = std::env::consts::ARCH;
    
    let os_name = match os {
        "windows" => "Windows",
        "macos" => "macOS",
        "linux" => "Linux",
        _ => os,
    };
    
    (os_name.to_string(), arch.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_file_size() {
        assert_eq!(format_file_size(1024), "1.00 KB");
        assert_eq!(format_file_size(1048576), "1.00 MB");
    }

    #[test]
    fn test_format_duration() {
        assert_eq!(format_duration(3661), "1h 1m 1s");
        assert_eq!(format_duration(125), "2m 5s");
    }
}
