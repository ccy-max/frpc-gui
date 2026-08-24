//! FRPC GUI - FRP 内网穿透桌面管理应用
//! 
//! 基于 Tauri v2 + Vue 3 开发
//! 支持 Windows/Linux/macOS 多平台

mod commands;
mod frp;
mod utils;

use commands::{init_app, *};
use log::info;
use tauri::Manager;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    // 初始化日志系统
    env_logger::Builder::from_env(
        env_logger::Env::default().default_filter_or("info")
    ).init();

    info!("Starting FRPC GUI application");

    tauri::Builder::default()
        // 插件配置
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_fs::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_notification::init())
        .plugin(tauri_plugin_autostart::init(
            tauri_plugin_autostart::MacosLauncher::LaunchAgent,
            Some(vec!["--autostart"]),
        ))
        // 系统托盘配置
        .setup(|app| {
            // 初始化应用状态
            init_app(app);
            
            // 设置系统托盘
            #[cfg(any(windows, target_os = "macos", target_os = "linux"))]
            {
                use tauri::{
                    menu::{Menu, MenuItem},
                    tray::{TrayIconBuilder, TrayIconEvent},
                };

                let show_i = MenuItem::with_id(app, "show", "显示主窗口", true, None::<&str>)?;
                let quit_i = MenuItem::with_id(app, "quit", "退出", true, None::<&str>)?;
                
                let menu = Menu::with_items(app, &[&show_i, &quit_i])?;
                
                let _tray = TrayIconBuilder::new()
                    .icon(app.default_window_icon().unwrap().clone())
                    .menu(&menu)
                    .on_menu_event(|app, event| match event.id.as_ref() {
                        "show" => {
                            if let Some(window) = app.get_webview_window("main") {
                                window.show().unwrap();
                                window.set_focus().unwrap();
                            }
                        }
                        "quit" => {
                            app.exit(0);
                        }
                        _ => {}
                    })
                    .build(app)?;
            }
            
            Ok(())
        })
        // IPC 命令处理
        .invoke_handler(tauri::generate_handler![
            // 配置管理
            commands::load_config,
            commands::save_config,
            commands::export_config,
            commands::import_config,
            // 进程控制
            commands::start_frp,
            commands::stop_frp,
            commands::restart_frp,
            commands::get_process_status,
            // 日志
            commands::get_logs,
            // 系统
            commands::check_frpc_exists,
            commands::get_frpc_version,
        ])
        // 运行应用
        .run(tauri::generate_context!())
        .expect("error while running FRPC GUI application");

    info!("FRPC GUI application exited");
}
