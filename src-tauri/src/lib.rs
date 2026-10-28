//! FRPC GUI - FRP 内网穿透桌面管理应用

// 预留功能/兼容层允许保留：导入导出、热重载、状态枚举变体等
// 当前未接前端但保留供后续启用，不应触发 dead_code 警告
#![allow(dead_code)]

mod commands;
mod frp;
mod utils;

use log::info;
use tauri::{Emitter, Manager};

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
                        .on_menu_event(|_app, event| match event.id.as_ref() {
                            "show" => {
                                if let Some(window) = _app.get_webview_window("main") {
                                    let _ = window.show();
                                    let _ = window.set_focus();
                                }
                            }
                            "quit" => {
                                // 托盘退出：必须"先杀 frpc 再退出"
                                // 历史 bug：这里只 emit 事件后立刻 _app.exit(0)，
                                // 前端根本来不及弹窗/清理，frpc 子进程全部残留。
                                // 改为：同步杀掉所有 frpc → 通知前端（尽力） → 退出
                                commands::kill_all_frpc_now(_app);
                                let _ = _app.emit("app-quit-requested", ());
                                _app.exit(0);
                            }
                            _ => {}
                        })
                        .build(app)?;
                } else {
                    log::warn!("No default window icon found, skipping tray icon");
                }
            }

            // 窗口关闭事件：通知前端弹对话框，由前端决定最小化还是退出
            //
            // 历史 bug（重要）：原实现用一个 `close_shown` 标志，置 true 后永不复位。
            // 用户第一次点关闭 → 弹窗；点 × 或"取消"关闭弹窗 → 标志仍是 true；
            // 用户再点关闭 → 走 return 直接放行 → 窗口销毁但 frpc 子进程继续运行（残留）。
            //
            // 正确做法：不用标志。每次 CloseRequested 都 prevent_close + 通知前端，
            // 由前端决定后续动作（exit_app 命令会先杀 frpc 再 process::exit，
            // 不再走 window.close()，从根本上绕开 CloseRequested 死锁）。
            if let Some(window) = app.get_webview_window("main") {
                let win = window.clone();
                window.on_window_event(move |event| match event {
                    tauri::WindowEvent::CloseRequested { api, .. } => {
                        // 每次关闭都拦截，交给前端弹窗决策
                        api.prevent_close();
                        let _ = win.emit("window-close-requested", ());
                    }
                    _ => {}
                });
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
            // 退出杀 frpc + 进程退出
            commands::kill_all_frpc_on_exit,
            commands::exit_app,
            commands::open_devtools,
            commands::send_notification,
            // N7 HTTP 请求检查器
            commands::inspector_start,
            commands::inspector_stop,
            commands::inspector_status,
            commands::inspector_list,
            commands::inspector_clear,
            commands::inspector_replay,
        ])
        .run(tauri::generate_context!())
        .unwrap_or_else(|e| {
            log::error!("致命错误：{}", e);
            std::process::exit(1);
        });
}
