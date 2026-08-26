//! FRP 进程管理模块
//!
//! 功能：进程启动/停止/重启/热重载、存活检测、自动守护、连接错误检测

use super::config::{ConfigManager, FrpConfig};
use anyhow::{Context, Result};
use log::{error, info, warn};
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::sync::atomic::{AtomicBool, AtomicU32, Ordering};
use std::sync::{Arc, Mutex};
use std::io::{BufRead, BufReader, Read};
use tokio::sync::mpsc;

#[cfg(windows)]
use std::os::windows::process::CommandExt;

/// 连接错误/成功匹配模式
const FRPC_ERROR_PATTERNS: &[&str] = &[
    "connect to server error",
    "login to server failed",
];
const FRPC_SUCCESS_PATTERNS: &[&str] = &[
    "login to server success",
    "start proxy success",
    "proxy added success",
];
/// 断线通知冷却时间（秒）
const DISCONNECT_COOLDOWN_SECS: u64 = 60;
/// 自动重启冷却时间（秒）
const RECOVERY_COOLDOWN_SECS: u64 = 10;

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
    child: Arc<Mutex<Option<std::process::Child>>>,
    log_tx: mpsc::Sender<String>,
    frpc_path: PathBuf,
    config_path: PathBuf,
    log_file_path: PathBuf,
    last_start_time: Arc<Mutex<i64>>,
    last_recovery_time: Arc<Mutex<i64>>,
    last_notify_time: Arc<Mutex<i64>>,
    recovery_checking: Arc<AtomicBool>,
    /// 该进程的 Admin API 端点 (addr, port, user, password)，启动时从配置快照
    admin_endpoint: std::sync::Mutex<Option<crate::frp::config::AdminConfig>>,
}

impl FrpProcessManager {
    pub fn new(frpc_path: PathBuf, config_path: PathBuf, log_tx: mpsc::Sender<String>) -> Self {
        let log_file_path = config_path.parent()
            .unwrap_or(&PathBuf::from("."))
            .join("frpc.log");

        Self {
            running: Arc::new(AtomicBool::new(false)),
            pid: Arc::new(AtomicU32::new(0)),
            child: Arc::new(Mutex::new(None)),
            log_tx,
            frpc_path,
            config_path,
            log_file_path,
            last_start_time: Arc::new(Mutex::new(-1)),
            last_recovery_time: Arc::new(Mutex::new(-1)),
            last_notify_time: Arc::new(Mutex::new(-1)),
            recovery_checking: Arc::new(AtomicBool::new(false)),
            admin_endpoint: std::sync::Mutex::new(None),
        }
    }

    /// 记录该进程的 Admin API 端点配置（启动时调用）
    pub fn set_admin_endpoint(&self, cfg: crate::frp::config::AdminConfig) {
        *self.admin_endpoint.lock().unwrap_or_else(|e| e.into_inner()) = Some(cfg);
    }

    /// 获取 Admin 端点；未配置时回退到默认值（127.0.0.1:7400 admin/admin）
    pub fn get_admin_endpoint(&self) -> crate::frp::config::AdminConfig {
        self.admin_endpoint
            .lock()
            .unwrap_or_else(|e| e.into_inner())
            .clone()
            .unwrap_or_default()
    }

    /// 启动 FRP 进程
    pub async fn start(&mut self, config: &FrpConfig) -> Result<()> {
        if self.is_process_alive() {
            return Err(anyhow::anyhow!("FRP 进程已在运行中 (PID: {})", self.pid()));
        }

        info!("Starting FRP process: {:?}", self.frpc_path);
        self.running.store(true, Ordering::SeqCst);

        // 生成 frpc.toml 配置文件（使用 generate_toml 排除 UI 字段）
        let cm = ConfigManager::new(self.config_path.clone());
        let toml_content = cm.generate_toml(config, self.log_file_path.to_string_lossy().as_ref())?;
        std::fs::write(&self.config_path, toml_content)
            .with_context(|| "写入 frpc.toml 失败")?;

        // 启动进程
        let mut cmd = Command::new(&self.frpc_path);
        cmd.arg("-c")
            .arg(&self.config_path)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped());

        #[cfg(windows)]
        {
            cmd.creation_flags(0x08000000); // CREATE_NO_WINDOW
        }

        match cmd.spawn() {
            Ok(mut child) => {
                let pid = child.id();
                info!("FRP process started with PID: {}", pid);

                self.pid.store(pid, Ordering::SeqCst);
                *self.last_start_time.lock().unwrap_or_else(|e| e.into_inner()) = chrono::Local::now().timestamp();

                // 提取 stdout/stderr
                let stdout = child.stdout.take();
                let stderr = child.stderr.take();

                // 保存子进程句柄
                *self.child.lock().unwrap_or_else(|e| e.into_inner()) = Some(child);

                // 启动日志捕获（stdout）
                if let Some(stdout) = stdout {
                    let log_tx = self.log_tx.clone();
                    tokio::spawn(async move {
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

                // 启动日志捕获（stderr）
                if let Some(stderr) = stderr {
                    let log_tx = self.log_tx.clone();
                    tokio::spawn(async move {
                        let reader = BufReader::new(stderr);
                        for line in reader.lines() {
                            match line {
                                Ok(text) => {
                                    if log_tx.send(format!("[FRP ERR] {}", text)).await.is_err() {
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
                self.running.store(false, Ordering::SeqCst);
                Err(anyhow::anyhow!("启动失败：{}", e))
            }
        }
    }

    /// 停止 FRP 进程
    pub async fn stop(&mut self) -> Result<()> {
        if !self.is_process_alive() {
            self.reset_state();
            return Ok(());
        }

        info!("Stopping FRP process, PID: {}", self.pid());
        let pid = self.pid();

        #[cfg(windows)]
        {
            Command::new("taskkill")
                .args(["/F", "/T", "/PID"])
                .arg(pid.to_string())
                .creation_flags(0x08000000)
                .output()
                .ok();
        }

        #[cfg(unix)]
        {
            // 先 SIGTERM，再 SIGKILL
            Command::new("kill")
                .arg("-TERM")
                .arg(pid.to_string())
                .output()
                .ok();
            
            std::thread::sleep(std::time::Duration::from_millis(500));
            
            if self.is_process_alive() {
                Command::new("kill")
                    .arg("-KILL")
                    .arg(pid.to_string())
                    .output()
                    .ok();
            }
        }

        // 关闭子进程句柄
        if let Some(mut child) = self.child.lock().unwrap_or_else(|e| e.into_inner()).take() {
            child.kill().ok();
            child.wait().ok();
        }

        self.reset_state();
        info!("FRP process stopped");
        Ok(())
    }

    /// 重启 FRP 进程
    pub async fn restart(&mut self, config: &FrpConfig) -> Result<()> {
        self.stop().await?;
        tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;
        self.start(config).await
    }

    /// 热重载配置（不重启进程）
    pub async fn reload(&self, config: &FrpConfig) -> Result<()> {
        if !self.is_process_alive() {
            return Err(anyhow::anyhow!("FRP 进程未运行，无法热重载"));
        }

        info!("Reloading FRP config, PID: {}", self.pid());

        // 重新生成配置文件
        let cm = ConfigManager::new(self.config_path.clone());
        let toml_content = cm.generate_toml(config, self.log_file_path.to_string_lossy().as_ref())?;
        std::fs::write(&self.config_path, toml_content)?;

        // 执行 frpc reload -c config_path
        let output = Command::new(&self.frpc_path)
            .arg("reload")
            .arg("-c")
            .arg(&self.config_path)
            .output()
            .with_context(|| "执行 frpc reload 失败")?;

        if output.status.success() {
            info!("FRP config reloaded successfully");
            Ok(())
        } else {
            let stderr = String::from_utf8_lossy(&output.stderr);
            Err(anyhow::anyhow!("热重载失败: {}", stderr))
        }
    }

    // ==================== 进程存活检测 ====================

    /// 检查进程是否存活（使用 kill(pid, 0) 方式）
    pub fn is_process_alive(&self) -> bool {
        let pid = self.pid.load(Ordering::SeqCst);
        if pid == 0 {
            return false;
        }

        #[cfg(unix)]
        {
            // 信号 0 = 检查进程是否存在，不实际发送信号
            use std::process::Command;
            let result = Command::new("kill")
                .arg("-0")
                .arg(pid.to_string())
                .output();
            match result {
                Ok(output) => output.status.success(),
                Err(_) => false,
            }
        }

        #[cfg(windows)]
        {
            // Windows: 用 tasklist 检查进程是否存在
            use std::process::Command;
            let output = Command::new("tasklist")
                .args(["/FI", &format!("PID eq {}", pid), "/FO", "CSV", "/NH"])
                .output();
            match output {
                Ok(o) => {
                    let stdout = String::from_utf8_lossy(&o.stdout);
                    stdout.contains(&pid.to_string())
                }
                Err(_) => false,
            }
        }
    }

    /// 探测外部 frpc 进程（应用重启后恢复状态）
    pub fn detect_external_process(&mut self) -> bool {
        #[cfg(windows)]
        {
            let frpc_name = "frpc.exe";
            let output = Command::new("tasklist")
                .args(["/FI", &format!("IMAGENAME eq {}", frpc_name), "/FO", "CSV", "/NH"])
                .output();
            if let Ok(o) = output {
                let stdout = String::from_utf8_lossy(&o.stdout);
                for line in stdout.lines() {
                    let parts: Vec<&str> = line.split("\",\"").collect();
                    if parts.len() >= 2 {
                        if let Ok(pid) = parts[1].parse::<u32>() {
                            self.pid.store(pid, Ordering::SeqCst);
                            self.running.store(true, Ordering::SeqCst);
                            *self.last_start_time.lock().unwrap_or_else(|e| e.into_inner()) = chrono::Local::now().timestamp();
                            info!("Detected external frpc process, PID: {}", pid);
                            return true;
                        }
                    }
                }
            }
        }

        #[cfg(unix)]
        {
            let frpc_name = "frpc";
            let output = Command::new("pgrep")
                .arg("-x")
                .arg(frpc_name)
                .output();
            if let Ok(o) = output {
                let stdout = String::from_utf8_lossy(&o.stdout);
                if let Some(first_line) = stdout.lines().next() {
                    if let Ok(pid) = first_line.trim().parse::<u32>() {
                        self.pid.store(pid, Ordering::SeqCst);
                        self.running.store(true, Ordering::SeqCst);
                        *self.last_start_time.lock().unwrap_or_else(|e| e.into_inner()) = chrono::Local::now().timestamp();
                        info!("Detected external frpc process, PID: {}", pid);
                        return true;
                    }
                }
            }
        }

        false
    }

    // ==================== 连接错误检测 ====================

    /// 读取日志文件尾部，检测连接错误
    pub fn check_connection_error(&self) -> Option<String> {
        if !self.log_file_path.exists() {
            return None;
        }

        let start_time = *self.last_start_time.lock().unwrap_or_else(|e| e.into_inner());
        if start_time == -1 {
            return None;
        }

        let file_size = std::fs::metadata(&self.log_file_path).ok()?.len();
        if file_size == 0 {
            return None;
        }

        let read_size = std::cmp::min(file_size, 8192) as usize;
        let mut file = std::fs::File::open(&self.log_file_path).ok()?;
        std::io::Seek::seek(&mut file, std::io::SeekFrom::End(-(read_size as i64))).ok()?;
        let mut buf = vec![0u8; read_size];
        file.read_exact(&mut buf).ok()?;
        let content = String::from_utf8_lossy(&buf);
        let lines: Vec<&str> = content.lines().filter(|l| !l.trim().is_empty()).collect();

        // 反向扫描
        for line in lines.iter().rev() {
            // 先检查成功模式（如果在错误之后出现，说明已重连）
            if FRPC_SUCCESS_PATTERNS.iter().any(|p| line.contains(*p)) {
                return None;
            }
            // 检查错误模式
            if FRPC_ERROR_PATTERNS.iter().any(|p| line.contains(*p)) {
                return Some(line.trim().to_string());
            }
        }

        None
    }

    // ==================== 进程守护 ====================

    /// 启动进程守护（定时检查 + 自动重启）
    pub fn start_guardian(self) {
        let pm = Arc::new(self.clone_shallow());
        tokio::spawn(async move {
            loop {
                tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;

                let running = pm.is_process_alive();
                let start_time = *pm.last_start_time.lock().unwrap_or_else(|e| e.into_inner());

                if !running && start_time != -1 {
                    // 检查冷却期
                    let now = chrono::Local::now().timestamp();
                    let last_recovery = *pm.last_recovery_time.lock().unwrap_or_else(|e| e.into_inner());
                    if last_recovery != -1 && (now - last_recovery) < RECOVERY_COOLDOWN_SECS as i64 {
                        continue;
                    }

                    if pm.recovery_checking.swap(true, Ordering::SeqCst) {
                        continue;
                    }

                    *pm.last_recovery_time.lock().unwrap_or_else(|e| e.into_inner()) = now;

                    // 检查网络
                    if check_internet().await {
                        warn!("FRP process died, network available, attempting restart...");
                        // 自动重启需要配置，这里只标记状态
                        let _ = pm.log_tx.send("[GUARD] 检测到进程退出，网络可用，等待自动重启".to_string()).await;
                    } else {
                        warn!("FRP process died, network unreachable, waiting...");
                        let _ = pm.log_tx.send("[GUARD] 检测到进程退出，网络不可用".to_string()).await;
                    }

                    pm.recovery_checking.store(false, Ordering::SeqCst);
                }
            }
        });
    }

    // ==================== 端口检查 ====================

    /// 检查端口是否被占用
    pub fn check_port_available(port: u16) -> bool {
        use std::net::TcpListener;
        TcpListener::bind(format!("127.0.0.1:{}", port)).is_ok()
    }

    // ==================== 状态获取 ====================

    /// 获取进程状态
    pub fn get_state(&self) -> ProcessState {
        if self.is_process_alive() {
            ProcessState::Running { pid: self.pid() }
        } else if self.running.load(Ordering::SeqCst) {
            // running flag 为 true 但进程已死
            self.reset_state();
            ProcessState::Stopped
        } else {
            ProcessState::Stopped
        }
    }

    /// 是否运行中
    pub fn is_running(&self) -> bool {
        self.is_process_alive()
    }

    /// 获取 PID（公开版本）
    pub fn get_pid(&self) -> u32 {
        self.pid.load(Ordering::SeqCst)
    }

    /// 获取 PID
    fn pid(&self) -> u32 {
        self.pid.load(Ordering::SeqCst)
    }

    /// 重置状态
    fn reset_state(&self) {
        self.running.store(false, Ordering::SeqCst);
        self.pid.store(0, Ordering::SeqCst);
        *self.last_start_time.lock().unwrap_or_else(|e| e.into_inner()) = -1;
        self.recovery_checking.store(false, Ordering::SeqCst);
    }

    /// 浅克隆（共享 Arc 内部状态）
    fn clone_shallow(&self) -> FrpProcessManager {
        FrpProcessManager {
            running: self.running.clone(),
            pid: self.pid.clone(),
            child: self.child.clone(),
            log_tx: self.log_tx.clone(),
            frpc_path: self.frpc_path.clone(),
            config_path: self.config_path.clone(),
            log_file_path: self.log_file_path.clone(),
            last_start_time: self.last_start_time.clone(),
            last_recovery_time: self.last_recovery_time.clone(),
            last_notify_time: self.last_notify_time.clone(),
            recovery_checking: self.recovery_checking.clone(),
            admin_endpoint: std::sync::Mutex::new(
                self.admin_endpoint.lock().unwrap_or_else(|e| e.into_inner()).clone(),
            ),
        }
    }
}

/// 检查网络连通性
async fn check_internet() -> bool {
    // 使用 std::process::Command 同步检查（简单可靠）
    #[cfg(windows)]
    {
        let output = Command::new("ping")
            .args(["-n", "1", "8.8.8.8"])
            .output();
        match output {
            Ok(o) => o.status.success(),
            Err(_) => false,
        }
    }

    #[cfg(unix)]
    {
        let output = Command::new("ping")
            .args(["-c", "1", "-W", "3", "8.8.8.8"])
            .output();
        match output {
            Ok(o) => o.status.success(),
            Err(_) => false,
        }
    }
}

// ==================== 独立函数 ====================

/// 检查 FRP 二进制文件是否存在
pub fn check_frpc_exists(path: &Path) -> bool {
    path.exists() && path.is_file()
}

/// 获取 FRP 版本
pub fn get_frpc_version(path: &Path) -> Result<String> {
    let output = Command::new(path)
        .arg("-v")
        .output()
        .with_context(|| "执行 frpc -v 失败")?;
    let version = String::from_utf8_lossy(&output.stdout);
    Ok(version.trim().to_string())
}
