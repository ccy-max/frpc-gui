//! FRP 进程管理模块

use super::config::FrpConfig;
use anyhow::{Context, Result};
use log::{error, info};
use std::path::{Path, PathBuf};
use std::process::{Child, Command, Stdio};
use std::sync::atomic::{AtomicBool, AtomicU32, Ordering};
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
    running: Arc<AtomicBool>,
    pid: Arc<AtomicU32>,
    child: Arc<Mutex<Option<Child>>>,
    log_tx: mpsc::Sender<String>,
    frpc_path: PathBuf,
    config_path: PathBuf,
}

impl FrpProcessManager {
    pub fn new(frpc_path: PathBuf, config_path: PathBuf, log_tx: mpsc::Sender<String>) -> Self {
        Self {
            running: Arc::new(AtomicBool::new(false)),
            pid: Arc::new(AtomicU32::new(0)),
            child: Arc::new(Mutex::new(None)),
            log_tx,
            frpc_path,
            config_path,
        }
    }

    /// 启动 FRP 进程
    pub async fn start(&mut self, config: &FrpConfig) -> Result<()> {
        if self.running.load(Ordering::SeqCst) {
            return Err(anyhow::anyhow!("FRP 进程已在运行中"));
        }

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
            Ok(mut child) => {
                let pid = child.id();
                info!("FRP process started with PID: {}", pid);

                self.running.store(true, Ordering::SeqCst);
                self.pid.store(pid, Ordering::SeqCst);

                // 提取 stdout 和 stderr 用于异步读取
                let stdout = child.stdout.take();
                let stderr = child.stderr.take();

                // 保存子进程句柄
                *self.child.lock().unwrap() = Some(child);

                // 启动日志捕获任务（使用标准线程读取管道）
                if let Some(stdout) = stdout {
                    let log_tx = self.log_tx.clone();
                    tokio::spawn(async move {
                        use std::io::{BufRead, BufReader};
                        let reader = BufReader::new(stdout);
                        for line in reader.lines() {
                            match line {
                                Ok(text) => {
                                    if log_tx.send(format!("[FRP] {}", text)).await.is_err() {
                                        break;
                                    }
                                }
                                Err(_) => break,
                            }
                        }
                    });
                }

                if let Some(stderr) = stderr {
                    let log_tx = self.log_tx.clone();
                    tokio::spawn(async move {
                        use std::io::{BufRead, BufReader};
                        let reader = BufReader::new(stderr);
                        for line in reader.lines() {
                            match line {
                                Ok(text) => {
                                    if log_tx.send(format!("[FRP] {}", text)).await.is_err() {
                                        break;
                                    }
                                }
                                Err(_) => break,
                            }
                        }
                    });
                }

                Ok(())
            }
            Err(e) => {
                error!("Failed to start FRP process: {}", e);
                Err(anyhow::anyhow!("启动失败：{}", e))
            }
        }
    }

    /// 停止 FRP 进程
    pub async fn stop(&mut self) -> Result<()> {
        if !self.running.load(Ordering::SeqCst) {
            return Ok(());
        }

        info!("Stopping FRP process");

        let pid = self.pid.load(Ordering::SeqCst);

        #[cfg(windows)]
        {
            if pid > 0 {
                Command::new("taskkill")
                    .args(["/F", "/T", "/PID"])
                    .arg(pid.to_string())
                    .output()
                    .ok();
            }
        }

        #[cfg(unix)]
        {
            if pid > 0 {
                Command::new("kill")
                    .arg("-TERM")
                    .arg(pid.to_string())
                    .output()
                    .ok();
            }
        }

        // 关闭子进程句柄
        if let Some(mut child) = self.child.lock().unwrap().take() {
            child.kill().ok();
            child.wait().ok();
        }

        self.running.store(false, Ordering::SeqCst);
        self.pid.store(0, Ordering::SeqCst);

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
        if self.running.load(Ordering::SeqCst) {
            let pid = self.pid.load(Ordering::SeqCst);
            ProcessState::Running { pid }
        } else {
            ProcessState::Stopped
        }
    }

    /// 检查进程是否运行
    pub fn is_running(&self) -> bool {
        self.running.load(Ordering::SeqCst)
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
