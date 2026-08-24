//! FRP 进程管理模块

use super::config::FrpConfig;
use anyhow::{Context, Result};
use log::{error, info, warn};
use std::path::{Path, PathBuf};
use std::process::{Child, Command, Stdio};
use std::sync::{Arc, Mutex};
use tokio::sync::mpsc;

/// 进程状态
#[derive(Debug, Clone)]
pub enum ProcessState {
    Stopped,
    Starting,
    Running { pid: u32 },
    Stopping,
    Error(String),
}

/// FRP 进程管理器
pub struct FrpProcessManager {
    child: Option<Arc<Mutex<Child>>>,
    state: Arc<Mutex<ProcessState>>,
    log_tx: mpsc::Sender<String>,
    frpc_path: PathBuf,
    config_path: PathBuf,
}

impl FrpProcessManager {
    pub fn new(frpc_path: PathBuf, config_path: PathBuf, log_tx: mpsc::Sender<String>) -> Self {
        Self {
            child: None,
            state: Arc::new(Mutex::new(ProcessState::Stopped)),
            log_tx,
            frpc_path,
            config_path,
        }
    }

    /// 启动 FRP 进程
    pub async fn start(&mut self, config: &FrpConfig) -> Result<()> {
        let mut state = self.state.lock().unwrap();
        
        if matches!(*state, ProcessState::Running { .. }) {
            return Err(anyhow::anyhow!("FRP 进程已在运行中"));
        }

        *state = ProcessState::Starting;
        drop(state);

        info!("Starting FRP process: {:?}", self.frpc_path);

        // 生成临时配置文件
        let temp_config_path = self.config_path.with_extension("tmp.toml");
        let config_manager = super::config::ConfigManager::new(temp_config_path.clone());
        config_manager.save(config)?;

        // 启动进程
        let mut cmd = Command::new(&self.frpc_path);
        cmd.arg("-c")
            .arg(&temp_config_path)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped());

        match cmd.spawn() {
            Ok(child) => {
                let pid = child.id();
                info!("FRP process started with PID: {}", pid);
                
                self.child = Some(Arc::new(Mutex::new(child)));
                
                // 更新状态
                *self.state.lock().unwrap() = ProcessState::Running { pid };
                
                // 启动日志捕获
                if let Some(child_arc) = self.child.clone() {
                    let log_tx = self.log_tx.clone();
                    tokio::spawn(async move {
                        capture_logs(child_arc, log_tx).await;
                    });
                }

                Ok(())
            }
            Err(e) => {
                error!("Failed to start FRP process: {}", e);
                *self.state.lock().unwrap() = ProcessState::Error(format!("启动失败：{}", e));
                Err(anyhow::anyhow!("Failed to start FRP: {}", e))
            }
        }
    }

    /// 停止 FRP 进程
    pub async fn stop(&mut self) -> Result<()> {
        let mut state = self.state.lock().unwrap();
        
        if matches!(*state, ProcessState::Stopped) {
            return Ok(());
        }

        *state = ProcessState::Stopping;
        drop(state);

        info!("Stopping FRP process");

        if let Some(child_arc) = self.child.take() {
            match Arc::try_unwrap(child_arc) {
                Ok(mutex_child) => {
                    let mut child = mutex_child.into_inner().unwrap();
                    #[cfg(windows)]
                    {
                        // Windows: 使用 taskkill 强制结束
                        use std::process::Command;
                        Command::new("taskkill")
                            .args(["/F", "/T", "/PID"])
                            .arg(child.id().to_string())
                            .output()
                            .ok();
                    }
                    #[cfg(unix)]
                    {
                        use nix::sys::signal::{kill, Signal};
                        use nix::unistd::Pid;
                        kill(Pid::from_raw(child.id() as i32), Signal::SIGTERM).ok();
                    }
                    
                    child.kill().ok();
                    child.wait().ok();
                    info!("FRP process stopped");
                }
                Err(_) => {
                    warn!("Failed to unwrap child Arc");
                }
            }
        }

        *self.state.lock().unwrap() = ProcessState::Stopped;
        Ok(())
    }

    /// 重启 FRP 进程
    pub async fn restart(&mut self, config: &FrpConfig) -> Result<()> {
        self.stop().await?;
        tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;
        self.start(config).await
    }

    /// 获取进程状态
    pub fn get_state(&self) -> ProcessState {
        self.state.lock().unwrap().clone()
    }

    /// 检查进程是否运行
    pub fn is_running(&self) -> bool {
        matches!(self.state.lock().unwrap().clone(), ProcessState::Running { .. })
    }
}

/// 捕获进程日志
async fn capture_logs(child_arc: Arc<Mutex<Child>>, log_tx: mpsc::Sender<String>) {
    use tokio::io::{AsyncBufReadExt, BufReader};
    use tokio::process::Child;
    
    // 重新打开子进程以捕获输出
    // 注意：这里需要重新获取子进程的 stdout/stderr
    // 实际实现中需要在启动时保存文件描述符
    
    // 简化版本：定期发送状态更新
    loop {
        {
            let mut child = child_arc.lock().unwrap();
            match child.try_wait() {
                Ok(Some(status)) => {
                    let _ = log_tx.send(format!("[FRP] 进程已退出，状态码：{}", status)).await;
                    break;
                }
                Ok(None) => {
                    // 进程仍在运行
                }
                Err(e) => {
                    let _ = log_tx.send(format!("[FRP] 检查进程状态失败：{}", e)).await;
                    break;
                }
            }
        }
        tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
    }
}

/// 检查 FRP 二进制文件是否存在
pub fn check_frpc_exists(path: &Path) -> bool {
    path.exists() && path.is_file()
}

/// 获取 FRP 版本
pub fn get_frpc_version(path: &Path) -> Result<String> {
    let output = Command::new(path)
        .arg("-v")
        .output()
        .with_context(|| "Failed to execute frpc -v")?;
    
    let version = String::from_utf8_lossy(&output.stdout);
    Ok(version.trim().to_string())
}
