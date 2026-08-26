//! 工具函数模块
//!
//! 本模块为预留公共工具库，部分函数当前未被调用，
//! 统一豁免 dead_code 警告（保留以供未来功能复用）。
#![allow(dead_code)]

pub mod settings;

use anyhow::Result;
use std::path::PathBuf;
use std::process::Command;

/// Windows 下隐藏控制台窗口（CREATE_NO_WINDOW = 0x08000000）
///
/// 背景：GUI 应用中所有子进程（tasklist/ping/netstat/powershell/frpc 等）
/// 默认会弹出 cmd 窗口，尤其是进程守护线程周期执行 tasklist/ping 时
/// 反复闪窗。所有子进程创建后必须调用本函数。
#[cfg(windows)]
pub fn hide_window(cmd: &mut Command) {
    use std::os::windows::process::CommandExt;
    const CREATE_NO_WINDOW: u32 = 0x0800_0000;
    cmd.creation_flags(CREATE_NO_WINDOW);
}

/// 非 Windows 平台空实现（保持调用点代码统一）
#[cfg(not(windows))]
pub fn hide_window(_cmd: &mut Command) {}

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
    let dir = get_app_data_dir()?.join("logs");
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
pub fn is_port_available(port: u16) -> bool {
    use std::net::TcpListener;
    TcpListener::bind(format!("127.0.0.1:{}", port)).is_ok()
}

/// 获取平台信息
pub fn get_platform_info() -> (String, String) {
    let os = std::env::consts::OS;
    let arch = std::env::consts::ARCH;
    (os.to_string(), arch.to_string())
}
