//! FRPC GUI - FRP 内网穿透桌面管理应用

// 预留功能/兼容层允许保留：导入导出、热重载、状态枚举变体等
// 当前未接前端但保留供后续启用，不应触发 dead_code 警告
#![allow(dead_code)]

mod commands;
mod frp;
mod utils;

use log::info;
use std::path::PathBuf;
use std::process::Command;
use tauri::Manager;

// Windows 下 Command::creation_flags 需要 CommandExt trait 在作用域内
#[cfg(windows)]
use std::os::windows::process::CommandExt;

/// 应用退出时清理所有 frpc 子进程（含孤儿），避免退出后进程残留
fn kill_all_frpc_on_exit(app: &tauri::AppHandle) {
    info!("App exiting, cleaning up frpc subprocesses");

    // 用 app.path() 获取配置目录（与 init_app 一致，而非 dirs::config_dir()）
    let config_dir = app.path().app_config_dir()
        .unwrap_or_else(|_| PathBuf::from("."))
        .join("frpc-gui");

    // 扫描 servers/*/frpc.pid，按记录的 PID 精准终止（覆盖应用重启后的孤儿）
    let servers_dir = config_dir.join("servers");
    if let Ok(entries) = std::fs::read_dir(&servers_dir) {
        for entry in entries.flatten() {
            let pid_file = entry.path().join("frpc.pid");
            if let Ok(content) = std::fs::read_to_string(&pid_file) {
                if let Ok(pid) = content.trim().parse::<u32>() {
                    kill_pid_by_platform(pid);
                }
            }
        }
    }

    // 兜底：直接清除所有 frpc.exe（包括未被 pid 文件记录的进程）
    #[cfg(windows)]
    {
        Command::new("taskkill")
            .args(["/F", "/IM", "frpc.exe", "/T"])
            .creation_flags(0x08000000)
            .output()
            .ok();
    }
    #[cfg(unix)]
    {
        Command::new("pkill").args(["-9", "frpc"]).output().ok();
    }
}

fn kill_pid_by_platform(pid: u32) {
    #[cfg(windows)]
    {
        Command::new("taskkill")
            .args(["/F", "/T", "/PID", &pid.to_string()])
            .creation_flags(0x08000000)
            .output()
            .ok();
    }
    #[cfg(unix)]
    {
        Command::new("kill").args(["-9", &pid.to_string()]).output().ok();
    }
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    env_logger::Builder::from_env(
        env_logger::Env::default().default_filter_or("info")
    ).init();

    info!("Starting FRPC GUI application");

    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_fs::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_notification::init())
        .plugin(tauri_plugin_autostart::init(
            tauri_plugin_autostart::MacosLauncher::LaunchAgent,
            Some(vec!["--autostart"]),
        ))
        .setup(|app| {
            commands::init_app(app);

            // 系统托盘
            #[cfg(any(windows, target_os = "macos", target_os = "linux"))]
            {
                use tauri::{
                    menu::{Menu, MenuItem},
                    tray::TrayIconBuilder,
                };

                let show_i = MenuItem::with_id(app, "show", "显示主窗口", true, None::<&str>)?;
                let quit_i = MenuItem::with_id(app, "quit", "退出", true, None::<&str>)?;
                let menu = Menu::with_items(app, &[&show_i, &quit_i])?;

                if let Some(icon) = app.default_window_icon() {
                    let _tray = TrayIconBuilder::new()
                        .icon(icon.clone())
                        .menu(&menu)
                        .on_menu_event(|app, event| match event.id.as_ref() {
                            "show" => {
                                if let Some(window) = app.get_webview_window("main") {
                                    let _ = window.show();
                                    let _ = window.set_focus();
                                }
                            }
                            "quit" => {
                                // 退出前清理所有 frpc 子进程（含孤儿），避免残留（S3 修复）
                                kill_all_frpc_on_exit(app);
                                app.exit(0);
                            }
                            _ => {}
                        })
                        .build(app)?;
                } else {
                    log::warn!("No default window icon found, skipping tray icon");
                }
            }
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            // 设置持久化
            commands::load_settings,
            commands::save_settings,
            // 配置管理（全局 UI 配置）
            commands::load_config,
            commands::save_config,
            commands::import_toml_config,
            commands::reset_all_config,
            // 进程控制
            commands::start_frp,
            commands::stop_frp,
            commands::restart_frp,
            commands::reload_frp,
            commands::get_process_status,
            commands::detect_frpc_process,
            commands::modify_proxy_status,
            // 多服务器进程控制
            commands::start_server,
            commands::stop_server,
            commands::restart_server,
            commands::get_server_status,
            commands::get_all_servers_status,
            commands::get_all_proxy_status,
            commands::get_server_traffic,
            commands::get_traffic_history,
            commands::get_connection_history,
            commands::log_connection_event,
            // 日志
            commands::get_logs,
            commands::get_frpc_log_content,
            commands::get_app_log_content,
            commands::open_frpc_log_file,
            // FRP 版本管理
            commands::list_frp_versions,
            commands::download_frp_version,
            commands::delete_frp_version,
            commands::set_active_version,
            commands::get_active_version,
            commands::get_mirrors,
            commands::import_local_frpc,
            commands::get_downloaded_versions,
            // 系统
            commands::check_frpc_exists,
            commands::get_frpc_version,
            commands::open_url,
            commands::relaunch_app,
            commands::open_app_data,
            commands::select_local_file,
            commands::check_app_update,
            commands::get_local_ports,
            // 持久化数据
            commands::load_persistent_data,
            commands::save_persistent_data,
        ])
        .run(tauri::generate_context!())
        .unwrap_or_else(|e| {
            log::error!("致命错误：{}", e);
            std::process::exit(1);
        });
}
