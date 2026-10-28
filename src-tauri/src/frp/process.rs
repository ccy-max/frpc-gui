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
use std::io::{BufRead, BufReader};
use tokio::sync::mpsc;

#[cfg(windows)]
use std::os::windows::process::CommandExt;

/// 连接错误/成功匹配模式

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
    recovery_checking: Arc<AtomicBool>,
    /// 该进程的 Admin API 端点 (addr, port, user, password)，启动时从配置快照
    admin_endpoint: std::sync::Mutex<Option<crate::frp::config::AdminConfig>>,
    /// 所属服务器 ID（实时日志事件推送时标识来源）
    server_id: std::sync::Mutex<Option<String>>,
    /// 进程 I/O 基准（启动时记录 GetProcessIoCounters 返回值，用于计算流量差值）
    /// (read_bytes, write_bytes)
    io_baseline: std::sync::Mutex<Option<(u64, u64)>>,
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
            recovery_checking: Arc::new(AtomicBool::new(false)),
            admin_endpoint: std::sync::Mutex::new(None),
            server_id: std::sync::Mutex::new(None),
            io_baseline: std::sync::Mutex::new(None),
        }
    }

    /// 设置所属服务器 ID（实时日志事件标识来源）
    pub fn set_server_id(&self, id: String) {
        *self.server_id.lock().unwrap_or_else(|e| e.into_inner()) = Some(id);
    }

    /// 读取所属服务器 ID
    fn get_server_id(&self) -> String {
        self.server_id
            .lock()
            .unwrap_or_else(|e| e.into_inner())
            .clone()
            .unwrap_or_else(|| "unknown".to_string())
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

    /// 获取进程 I/O 基准（启动时记录的 GetProcessIoCounters 值）
    pub fn get_io_baseline(&self) -> Option<(u64, u64)> {
        *self.io_baseline.lock().unwrap_or_else(|e| e.into_inner())
    }

    /// 记录进程 I/O 基准（启动时调用）
    pub fn set_io_baseline(&self, baseline: (u64, u64)) {
        *self.io_baseline.lock().unwrap_or_else(|e| e.into_inner()) = Some(baseline);
    }

    /// 启动 FRP 进程
    pub async fn start(&mut self, config: &FrpConfig) -> Result<()> {
        if self.is_process_alive() {
            return Err(anyhow::anyhow!("FRP 进程已在运行中 (PID: {})", self.pid()));
        }

        // 启动前清场：杀掉本服务器配置下的孤儿 frpc 进程
        // 历史 bug：应用重启/停止失败残留的孤儿进程与新实例叠加，
        // 旧进程用旧配置持续运行（连错端口、刷旧日志），用户误以为配置不生效
        // 性能：kill_orphans 内部已轮询确认进程退出，无需额外 sleep
        if self.is_any_orphan_alive() {
            let n = self.kill_orphans();
            warn!("Cleaned {} orphan frpc process(es) before start", n);
        }

        info!("Starting FRP process: {:?}", self.frpc_path);
        self.running.store(true, Ordering::SeqCst);

        // 分配 Admin/WebServer 端口：web_server.port==0 表示由后端自动分配，
        // 避免多进程同时运行都抢 7400 造成冲突（此前为规避冲突干脆关掉 Admin API，
        // 导致流量/代理状态监控整体失效——S4 修复）。
        let mut cfg = config.clone();
        if cfg.web_server.port == 0 {
            let mut p = 7400u16;
            while p < 7500 && !FrpProcessManager::check_port_available(p) {
                p += 1;
            }
            cfg.web_server.port = p;
        }
        // 记录 Admin API 端点（用分配后的端口，供监控查询使用）
        // frpc 的 Admin API 即 [webServer] 段，字段与 AdminConfig 同构
        self.set_admin_endpoint(crate::frp::config::AdminConfig {
            addr: cfg.web_server.addr.clone(),
            port: cfg.web_server.port,
            user: cfg.web_server.user.clone(),
            password: cfg.web_server.password.clone(),
        });

        // 生成配置文件：按 frpc 版本自动选择格式
        // - frp >= 0.52.0 → TOML（新版格式，serverAddr = "..."）
        // - frp <  0.52.0 → INI（旧版格式，[common] 段）
        // 历史 bug：旧版 frpc 读 TOML 报
        //   "invalid configuration file, not found [common] section"
        let cm = ConfigManager::new(self.config_path.clone());
        let use_toml = Self::frpc_supports_toml(&self.frpc_path);
        let config_file = if use_toml {
            let toml_content = cm.generate_toml(&cfg, self.log_file_path.to_string_lossy().as_ref())?;
            std::fs::write(&self.config_path, toml_content)
                .with_context(|| "写入 frpc.toml 失败")?;
            self.config_path.clone()
        } else {
            info!("frpc 为旧版本，使用 INI 格式配置");
            let ini_path = self.config_path.with_extension("ini");
            let ini_content = cm.generate_ini(&cfg, self.log_file_path.to_string_lossy().as_ref())?;
            std::fs::write(&ini_path, ini_content)
                .with_context(|| "写入 frpc.ini 失败")?;
            ini_path
        };

        // 启动进程
        let mut cmd = Command::new(&self.frpc_path);
        cmd.arg("-c")
            .arg(&config_file)
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
                self.save_pid(pid); // 持久化 PID（应用重启后 stop 仍能找到进程）
                *self.last_start_time.lock().unwrap_or_else(|e| e.into_inner()) = chrono::Local::now().timestamp();

                // 记录进程 I/O 基准（用于流量监控：GetProcessIoCounters 返回累计值，
                // 后续查询时用 current - baseline 得到运行期间流量）
                if let Some(io) = get_process_io_counters(pid) {
                    self.set_io_baseline(io);
                    info!("Recorded I/O baseline for PID {}: read={}, write={}", pid, io.0, io.1);
                }

                // 提取 stdout/stderr
                let stdout = child.stdout.take();
                let stderr = child.stderr.take();

                // 保存子进程句柄
                *self.child.lock().unwrap_or_else(|e| e.into_inner()) = Some(child);

                // 启动日志捕获（stdout）
                if let Some(stdout) = stdout {
                    let log_tx = self.log_tx.clone();
                    let sid = self.get_server_id();
                    tokio::spawn(async move {
                        let reader = BufReader::new(stdout);
                        for line in reader.lines() {
                            match line {
                                Ok(text) => {
                                    if log_tx.send(format!("[{}][FRP] {}", sid, text)).await.is_err() {
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
                    let sid = self.get_server_id();
                    tokio::spawn(async move {
                        let reader = BufReader::new(stderr);
                        for line in reader.lines() {
                            match line {
                                Ok(text) => {
                                    if log_tx.send(format!("[{}][FRP ERR] {}", sid, text)).await.is_err() {
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
    /// PID 记录文件路径（servers/{id}/frpc.pid）
    ///
    /// 历史 bug：应用重启后 process manager 重建（内存 pid=0），
    /// stop 时 is_process_alive()=false 直接假装停止成功，
    /// 孤儿 frpc 进程永远无人处理（用户任务管理器可见 frpc.exe 残留）。
    fn pid_file(&self) -> PathBuf {
        self.config_path.parent()
            .unwrap_or_else(|| Path::new("."))
            .join("frpc.pid")
    }

    /// 持久化 PID（spawn 成功后调用）
    fn save_pid(&self, pid: u32) {
        if let Some(dir) = self.pid_file().parent() {
            let _ = std::fs::create_dir_all(dir);
        }
        let _ = std::fs::write(self.pid_file(), pid.to_string());
    }

    /// 读取持久化的 PID
    fn load_pid(&self) -> Option<u32> {
        std::fs::read_to_string(self.pid_file()).ok()?
            .trim().parse::<u32>().ok().filter(|p| *p > 0)
    }

    /// 删除 PID 记录文件
    fn clear_pid_file(&self) {
        let _ = std::fs::remove_file(self.pid_file());
    }

    /// 按配置路径查找孤儿 frpc 进程 PID
    ///
    /// 精准匹配命令行中包含本服务器配置目录路径的 frpc 进程，
    /// 不会误杀其他实例或其他程序的 frpc。
    ///
    /// 性能：sysinfo 进程内快照（<50ms），替代旧 PowerShell Get-CimInstance
    /// 子进程方案（冷启动 1~3s，是启动慢的最大元凶）。
    fn find_orphan_pids_by_config(&self) -> Vec<u32> {
        let dir_marker = self.config_path.parent()
            .map(|p| p.to_string_lossy().to_string())
            .unwrap_or_default();
        if dir_marker.is_empty() {
            return vec![];
        }
        // Windows 路径大小写不敏感；统一小写比对
        let marker_l = dir_marker.to_lowercase();
        sys_list_frpc_procs()
            .into_iter()
            .filter(|(pid, cmd)| {
                *pid != std::process::id() && cmd.to_lowercase().contains(&marker_l)
            })
            .map(|(pid, _)| pid)
            .collect()
    }

    /// 清理本服务器配置目录下的所有孤儿 frpc 进程
    ///
    /// 返回杀掉的 PID 数量。start 前调用防止多实例叠加；
    /// stop 时兜底（应用重启后内存句柄丢失场景）。
    fn kill_orphans(&self) -> usize {
        let pids = self.find_orphan_pids_by_config();
        let count = pids.len();
        for pid in pids {
            info!("Killing orphan frpc process, PID: {}", pid);
            sys_kill_pid_tree(pid);
        }
        if count > 0 {
            // 等待进程退出：轮询确认（最多 500ms），替代定长 sleep
            for _ in 0..10 {
                if self.find_orphan_pids_by_config().is_empty() {
                    break;
                }
                std::thread::sleep(std::time::Duration::from_millis(50));
            }
        }
        count
    }

    pub async fn stop(&mut self) -> Result<()> {
        let mut pid = self.pid();

        // PID 为 0 时从 PID 文件恢复（应用重启后 manager 重建场景）
        if pid == 0 {
            pid = self.load_pid().unwrap_or(0);
        }

        if pid != 0 && self.is_process_alive_by_pid(pid) {
            info!("Stopping FRP process, PID: {}", pid);

            #[cfg(windows)]
            {
                sys_kill_pid_tree(pid);
            }

            #[cfg(unix)]
            {
                Command::new("kill").arg("-TERM").arg(pid.to_string()).output().ok();
                // 轮询等待优雅退出（最多 500ms），超时才强杀——替代定长 500ms sleep
                for _ in 0..10 {
                    if !self.is_process_alive_by_pid(pid) { break; }
                    std::thread::sleep(std::time::Duration::from_millis(50));
                }
                if self.is_process_alive_by_pid(pid) {
                    Command::new("kill").arg("-KILL").arg(pid.to_string()).output().ok();
                }
            }

            // 验证进程确实死亡：轮询（最多 300ms），死亡即刻继续
            for _ in 0..6 {
                if !self.is_process_alive_by_pid(pid) { break; }
                std::thread::sleep(std::time::Duration::from_millis(50));
            }
            if self.is_process_alive_by_pid(pid) {
                warn!("PID {} still alive after kill, forcing orphan cleanup", pid);
                self.kill_orphans();
            }
        } else if self.is_any_orphan_alive() {
            // 内存与 PID 文件都无有效进程，但存在同配置的孤儿进程
            // （应用重启后旧 frpc 仍在跑的场景——用户截图实锤）
            info!("No tracked PID but orphan frpc detected, cleaning up");
            self.kill_orphans();
        }

        // 关闭子进程句柄
        if let Some(mut child) = self.child.lock().unwrap_or_else(|e| e.into_inner()).take() {
            child.kill().ok();
            child.wait().ok();
        }

        self.clear_pid_file();
        self.reset_state();
        info!("FRP process stopped");
        Ok(())
    }

    /// 检查是否存在本配置下的存活 frpc 进程（孤儿检测）
    fn is_any_orphan_alive(&self) -> bool {
        !self.find_orphan_pids_by_config().is_empty()
    }

    /// 按 PID 检查进程存活（不依赖内存句柄）
    ///
    /// sysinfo 进程内快照，替代 tasklist/kill -0 子进程方案。
    fn is_process_alive_by_pid(&self, pid: u32) -> bool {
        if pid == 0 {
            return false;
        }
        sys_pid_alive(pid)
    }

    /// 重启 FRP 进程
    pub async fn restart(&mut self, config: &FrpConfig) -> Result<()> {
        self.stop().await?;
        // stop 内部已轮询确认进程退出；此处短暂等待端口释放即可
        tokio::time::sleep(tokio::time::Duration::from_millis(200)).await;
        self.start(config).await
    }

    /// 热重载配置（不重启进程）
    pub async fn reload(&self, config: &FrpConfig) -> Result<()> {
        if !self.is_process_alive() {
            return Err(anyhow::anyhow!("FRP 进程未运行，无法热重载"));
        }

        info!("Reloading FRP config, PID: {}", self.pid());

        // 旧版 frpc（< 0.52.0，INI 格式）不支持 `reload` 子命令且配置是 INI，
        // 热重载会静默失败——明确报错引导用户改用重启
        if !Self::frpc_supports_toml(&self.frpc_path) {
            return Err(anyhow::anyhow!(
                "当前 frpc 版本（< 0.52.0）不支持热重载，请在代理页使用「重启」代替"
            ));
        }

        // 重新生成配置文件
        let cm = ConfigManager::new(self.config_path.clone());
        let toml_content = cm.generate_toml(config, self.log_file_path.to_string_lossy().as_ref())?;
        std::fs::write(&self.config_path, toml_content)?;

        // 执行 frpc reload -c config_path
        let mut cmd = Command::new(&self.frpc_path);
        crate::utils::hide_window(&mut cmd);
        let output = cmd
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

    /// 检查进程是否存活（sysinfo 进程内快照，轮询高频调用零子进程开销）
    pub fn is_process_alive(&self) -> bool {
        let pid = self.pid.load(Ordering::SeqCst);
        if pid == 0 {
            return false;
        }
        sys_pid_alive(pid)
    }

    /// 探测外部 frpc 进程（应用重启后恢复状态）
    // ==================== 进程守护 ====================

    /// 启动进程守护（定时检查 + 自动重启）
    // ==================== 端口检查 ====================

    /// 检查端口是否被占用
    pub fn check_port_available(port: u16) -> bool {
        use std::net::TcpListener;
        TcpListener::bind(format!("127.0.0.1:{}", port)).is_ok()
    }

    // ==================== 状态获取 ====================

    /// 获取进程状态
    /// 获取进程启动时刻（Unix 秒级时间戳；未运行返回 None）
    pub fn get_started_at(&self) -> Option<i64> {
        let t = *self.last_start_time.lock().unwrap_or_else(|e| e.into_inner());
        if t > 0 { Some(t) } else { None }
    }

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

}

// ==================== 独立函数 ====================

/// frpc 版本缓存：路径 → (是否支持TOML, 版本号字符串)
///
/// 历史性能问题：每次 start/reload 都 spawn `frpc -v` 子进程（Windows 上 300ms~1s）。
/// frpc 二进制内容不变则版本不变，进程内缓存一次即可；
/// 切换激活版本会换路径 → 天然缓存失效；覆盖导入同路径版本由 set_active_version 调
/// invalidate_version_cache 显式清理。
static FRPC_TOML_CACHE: once_cell::sync::Lazy<Mutex<std::collections::HashMap<PathBuf, bool>>> =
    once_cell::sync::Lazy::new(|| Mutex::new(std::collections::HashMap::new()));

/// 清除 frpc 版本缓存（版本切换/覆盖导入时调用）
pub fn invalidate_version_cache() {
    if let Ok(mut m) = FRPC_TOML_CACHE.lock() {
        m.clear();
        info!("frpc version cache invalidated");
    }
}

// ==================== 进程内枚举（sysinfo，替代 PowerShell/tasklist 子进程） ====================

/// 按 PID 检查进程存活（进程内读系统快照，毫秒级，无子进程开销）
pub fn sys_pid_alive(pid: u32) -> bool {
    use sysinfo::{Pid, System};
    let mut sys = System::new();
    sys.refresh_processes();
    sys.process(Pid::from_u32(pid)).is_some()
}

/// 枚举所有 frpc 进程：返回 (pid, 命令行) 列表
///
/// 用于孤儿检测——替代 PowerShell Get-CimInstance（冷启动 1~3s）。
pub fn sys_list_frpc_procs() -> Vec<(u32, String)> {
    use sysinfo::System;
    let mut sys = System::new();
    sys.refresh_processes();
    let mut out = Vec::new();
    for (pid, proc_) in sys.processes() {
        let name = proc_.name().to_lowercase();
        if name == "frpc.exe" || name == "frpc" {
            let cmd = proc_.cmd().join(" ");
            out.push((pid.as_u32(), cmd));
        }
    }
    out
}

/// 强杀指定 PID（含进程树）——sysinfo 版本，无子进程
///
/// 0.30 API 无 children()：先快照全部进程，收集 parent==pid 的子进程，
/// 先杀子再杀父。
pub fn sys_kill_pid_tree(pid: u32) {
    use sysinfo::{Pid, System};
    let mut sys = System::new();
    sys.refresh_processes();
    let target = Pid::from_u32(pid);
    let children: Vec<Pid> = sys.processes().iter()
        .filter(|(_, p)| p.parent().map(|pp| pp == target).unwrap_or(false))
        .map(|(cpid, _)| *cpid)
        .collect();
    for c in children {
        if let Some(cp) = sys.process(c) {
            cp.kill();
        }
    }
    if let Some(p) = sys.process(target) {
        p.kill();
    }
}

/// 检查 FRP 二进制文件是否存在
pub fn check_frpc_exists(path: &Path) -> bool {
    path.exists() && path.is_file()
}

/// 获取 FRP 版本
pub fn get_frpc_version(path: &Path) -> Result<String> {
    let mut cmd = Command::new(path);
    crate::utils::hide_window(&mut cmd);
    let output = cmd
        .arg("-v")
        .output()
        .with_context(|| "执行 frpc -v 失败")?;
    let version = String::from_utf8_lossy(&output.stdout);
    Ok(version.trim().to_string())
}

impl FrpProcessManager {
    /// 检测指定 frpc 是否支持 TOML 配置格式
    ///
    /// frp v0.52.0 起引入 TOML/YAML/JSON 并弃用 INI；
    /// 更早版本只认 INI（[common] 段），读 TOML 会报
    /// "invalid configuration file, not found [common] section"。
    ///
    /// 版本获取失败时保守返回 true（假定新版，走 TOML）。
    pub fn frpc_supports_toml(frpc_path: &Path) -> bool {
        // 缓存命中直接返回，避免重复 spawn `frpc -v`
        if let Ok(m) = FRPC_TOML_CACHE.lock() {
            if let Some(&v) = m.get(frpc_path) {
                return v;
            }
        }
        let supports = match get_frpc_version(frpc_path) {
            Ok(ver) => {
                let v = ver.trim().trim_start_matches('v');
                let parts: Vec<u32> = v.split('.')
                    .map(|s| s.trim().parse::<u32>().unwrap_or(0))
                    .collect();
                let s = parts.first().copied().unwrap_or(0) > 0
                    || parts.get(1).copied().unwrap_or(0) >= 52;
                info!("frpc version {}: toml_support={} (cached)", ver, s);
                s
            }
            Err(e) => {
                warn!("获取 frpc 版本失败({})，假定支持 TOML", e);
                true
            }
        };
        if let Ok(mut m) = FRPC_TOML_CACHE.lock() {
            m.insert(frpc_path.to_path_buf(), supports);
        }
        supports
    }
}

/// 获取进程的累计 I/O 读/写字节数
///
/// Windows: 使用 GetProcessIoCounters 获取进程级 I/O 统计。
/// 返回的 (read_bytes, write_bytes) 包含进程的所有 I/O（网络 + 磁盘），
/// 但 frpc 进程以网络 I/O 为主（本地转发），磁盘 I/O 仅限日志写入（量小），
/// 因此可近似为网络流量。
///
/// 用法：启动时记录 baseline，查询时用 current - baseline 得到运行期间流量。
#[cfg(windows)]
pub fn get_process_io_counters(pid: u32) -> Option<(u64, u64)> {
    use windows::Win32::Foundation::CloseHandle;
    use windows::Win32::System::IO::IO_COUNTERS;
    use windows::Win32::System::ProcessStatus::GetProcessIoCounters;
    use windows::Win32::System::Threading::{OpenProcess, PROCESS_QUERY_LIMITED_INFORMATION};

    unsafe {
        let handle = OpenProcess(PROCESS_QUERY_LIMITED_INFORMATION, false, pid).ok()?;
        let mut counters: IO_COUNTERS = std::mem::zeroed();
        let result = GetProcessIoCounters(handle, std::ptr::addr_of_mut!(counters));
        let _ = CloseHandle(handle);
        result.ok()?;
        Some((counters.ReadTransferCount, counters.WriteTransferCount))
    }
}

/// 非 Windows 平台：不支持进程级 I/O 监控
#[cfg(not(windows))]
pub fn get_process_io_counters(_pid: u32) -> Option<(u64, u64)> {
    None
}
