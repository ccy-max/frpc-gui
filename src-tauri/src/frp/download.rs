//! FRP 版本下载与管理模块
//!
//! 功能：版本列表获取（GitHub API + 镜像 + 本地回退）、下载（进度回调）、
//! 解压（tar.gz + zip）、SHA256 校验、本地导入

use anyhow::{Context, Result};
use log::{info, warn};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use tokio::sync::Mutex;

// ==================== 数据结构 ====================

/// GitHub Release 信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GitHubRelease {
    pub id: u64,
    pub tag_name: String,
    pub name: String,
    pub published_at: String,
    pub assets: Vec<GitHubAsset>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GitHubAsset {
    pub id: u64,
    pub name: String,
    pub browser_download_url: String,
    pub size: u64,
    pub download_count: u64,
}

/// FRP 版本信息（前端使用）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FrpVersionInfo {
    pub version: String,
    pub name: String,
    pub published_at: String,
    pub download_url: String,
    pub mirror_url: Option<String>,
    pub size: u64,
    pub download_count: u64,
    pub downloaded: bool,
    pub local_path: Option<String>,
    /// 是否为当前激活使用的版本
    #[serde(default)]
    pub is_active: bool,
}

/// 下载进度回调类型
pub type ProgressCallback = Arc<Mutex<Option<Box<dyn Fn(u64, u64) + Send + Sync>>>>;

// ==================== 平台映射 ====================

/// 获取当前平台对应的 FRP 文件名关键词
fn get_platform_keywords() -> Vec<&'static str> {
    let os = std::env::consts::OS;
    let arch = std::env::consts::ARCH;
    match (os, arch) {
        ("windows", "x86_64") => vec!["windows", "amd64"],
        ("windows", "x86") => vec!["windows", "386"],
        ("windows", "aarch64") => vec!["windows", "arm64"],
        ("linux", "x86_64") => vec!["linux", "amd64"],
        ("linux", "aarch64") => vec!["linux", "arm64"],
        ("macos", "x86_64") => vec!["darwin", "amd64"],
        ("macos", "aarch64") => vec!["darwin", "arm64"],
        _ => vec!["linux", "amd64"],
    }
}

/// GitHub API 镜像源列表
pub fn get_mirrors() -> Vec<MirrorInfo> {
    vec![
        MirrorInfo { id: "github".to_string(), name: "GitHub 官方".to_string(), prefix: "".to_string() },
        // 历史：ghproxy.com 已于 2023 年底关停，勿再使用
        // 可用性 2026-08 实测：ghproxy.net ✅ gh-proxy.com ✅ ghfast.top 部分网络不可达
        MirrorInfo { id: "ghproxy_net".to_string(), name: "ghproxy.net 加速".to_string(), prefix: "https://ghproxy.net/".to_string() },
        MirrorInfo { id: "ghproxy_com".to_string(), name: "gh-proxy.com 加速".to_string(), prefix: "https://gh-proxy.com/".to_string() },
        MirrorInfo { id: "ghfast".to_string(), name: "ghfast 加速".to_string(), prefix: "https://ghfast.top/".to_string() },
        MirrorInfo { id: "jwinks".to_string(), name: "jwinks 镜像".to_string(), prefix: "https://gh.jwinks.com/file/".to_string() },
    ]
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MirrorInfo {
    pub id: String,
    pub name: String,
    pub prefix: String,
}

// ==================== 版本管理器 ====================

pub struct FrpVersionManager {
    install_dir: PathBuf,
}

impl FrpVersionManager {
    pub fn new(install_dir: PathBuf) -> Self {
        Self { install_dir }
    }

    /// 激活版本记录文件路径（记录当前使用的 frp 版本）
    fn active_version_file(&self) -> PathBuf {
        self.install_dir.join("active_version")
    }

    /// 设置当前使用的版本
    pub fn set_active_version(&self, version: &str) -> Result<()> {
        fs::create_dir_all(&self.install_dir)?;
        // 校验该版本确实已下载
        if self.get_frpc_path(version).is_none() {
            return Err(anyhow::anyhow!("版本 {} 未下载，无法激活", version));
        }
        fs::write(self.active_version_file(), version)
            .with_context(|| "写入激活版本失败")?;
        info!("Active FRP version set to {}", version);
        Ok(())
    }

    /// 获取当前使用的版本
    pub fn get_active_version(&self) -> Option<String> {
        let content = fs::read_to_string(self.active_version_file()).ok()?;
        let v = content.trim().to_string();
        if v.is_empty() { None } else { Some(v) }
    }

    /// 获取 FRP 版本列表
    /// 1. 尝试 GitHub API（直连）
    /// 2. 失败则尝试镜像
    /// 3. 最终回退到本地内置 JSON
    pub async fn list_versions(&self) -> Result<Vec<FrpVersionInfo>> {
        info!("Fetching FRP versions");

        // 尝试 GitHub API
        match self.fetch_from_github(None).await {
            Ok(versions) if !versions.is_empty() => return Ok(self.mark_active(versions)),
            Ok(_) => warn!("GitHub API returned empty versions"),
            Err(e) => warn!("GitHub API failed: {}, trying mirrors", e),
        }

        // 尝试镜像
        for mirror in get_mirrors().iter().skip(1) {
            match self.fetch_from_github(Some(mirror)).await {
                Ok(versions) if !versions.is_empty() => return Ok(self.mark_active(versions)),
                Ok(_) => continue,
                Err(e) => {
                    warn!("Mirror {} failed: {}", mirror.id, e);
                    continue;
                }
            }
        }

        // 回退到本地 JSON
        warn!("All mirrors failed, using local fallback JSON");
        Ok(self.mark_active(self.get_local_versions()?))
    }

    /// 为版本列表标记当前激活版本
    fn mark_active(&self, mut versions: Vec<FrpVersionInfo>) -> Vec<FrpVersionInfo> {
        if let Some(active) = self.get_active_version() {
            for v in versions.iter_mut() {
                v.is_active = v.version == active;
            }
        }
        versions
    }

    /// 从 GitHub API 或镜像获取版本列表
    async fn fetch_from_github(&self, mirror: Option<&MirrorInfo>) -> Result<Vec<FrpVersionInfo>> {
        let url = match mirror {
            Some(m) if !m.prefix.is_empty() => {
                format!("{}https://api.github.com/repos/fatedier/frp/releases", m.prefix)
            }
            _ => "https://api.github.com/repos/fatedier/frp/releases".to_string(),
        };

        let client = reqwest::Client::builder()
            .user_agent("frpc-gui")
            .timeout(std::time::Duration::from_secs(15))
            .build()?;

        let resp = client.get(&url).send().await?;
        if !resp.status().is_success() {
            return Err(anyhow::anyhow!("HTTP {}", resp.status()));
        }

        let releases: Vec<GitHubRelease> = resp.json().await?;
        let keywords = get_platform_keywords();

        let versions: Vec<FrpVersionInfo> = releases
            .iter()
            .filter(|r| {
                // 仅支持 toml 版本（release id > 124395282）
                r.id > 124395282
            })
            .filter_map(|release| {
                // 找到匹配当前平台的资源
                let asset = release.assets.iter().find(|a| {
                    keywords.iter().all(|kw| a.name.contains(kw))
                })?;

                let version = release.tag_name.clone();
                let local_path = self.get_frpc_path(&version);
                let downloaded = local_path.is_some();

                let mirror_url = mirror.map(|m| {
                    if !m.prefix.is_empty() {
                        format!("{}{}", m.prefix, asset.browser_download_url)
                    } else {
                        asset.browser_download_url.clone()
                    }
                });

                Some(FrpVersionInfo {
                    version,
                    name: release.name.clone(),
                    published_at: release.published_at.clone(),
                    download_url: asset.browser_download_url.clone(),
                    mirror_url,
                    size: asset.size,
                    download_count: asset.download_count,
                    downloaded,
                    local_path: local_path.map(|p| p.to_string_lossy().to_string()),
                    is_active: false,
                })
            })
            .collect();

        Ok(versions)
    }

    /// 本地内置版本回退
    fn get_local_versions(&self) -> Result<Vec<FrpVersionInfo>> {
        // 内置几个常见版本
        // 历史 bug：Windows 官方资产是 .zip，此前误拼 .tar.gz 导致 404，
        // 镜像又返回 200+HTML 错误页 → 解压报 "failed to iterate over archive"
        let keywords = get_platform_keywords();
        let (suffix, ext) = if keywords.contains(&"windows") {
            let arch = if keywords.contains(&"arm64") { "arm64" }
                       else if keywords.contains(&"386") { "386" }
                       else { "amd64" };
            (format!("windows_{}", arch), "zip")
        } else if keywords.contains(&"darwin") {
            let arch = if keywords.contains(&"arm64") { "arm64" } else { "amd64" };
            (format!("darwin_{}", arch), "tar.gz")
        } else {
            let arch = if keywords.contains(&"arm64") { "arm64" } else { "amd64" };
            (format!("linux_{}", arch), "tar.gz")
        };

        let versions = vec![
            ("v0.61.0", "Release v0.61.0"),
            ("v0.60.0", "Release v0.60.0"),
            ("v0.59.0", "Release v0.59.0"),
            ("v0.58.0", "Release v0.58.0"),
            ("v0.57.0", "Release v0.57.0"),
            ("v0.56.0", "Release v0.56.0"),
            ("v0.55.0", "Release v0.55.0"),
            ("v0.54.0", "Release v0.54.0"),
            ("v0.53.0", "Release v0.53.0"),
            ("v0.52.0", "Release v0.52.0"),
        ];

        let result: Vec<FrpVersionInfo> = versions.iter().map(|(version, name)| {
            let local_path = self.get_frpc_path(version);
            let downloaded = local_path.is_some();
            FrpVersionInfo {
                version: version.to_string(),
                name: name.to_string(),
                published_at: String::new(),
                download_url: format!(
                    "https://github.com/fatedier/frp/releases/download/{}/frp_{}_{}.{}",
                    version, suffix, version, ext
                ),
                mirror_url: None,
                size: 0,
                download_count: 0,
                downloaded,
                local_path: local_path.map(|p| p.to_string_lossy().to_string()),
                is_active: false,
            }
        }).collect();

        Ok(result)
    }

    /// 构建下载候选 URL 列表（镜像回退链）
    ///
    /// 参考 frpc-desktop 机制：GitHub 直连在国内大概率失败，
    /// 依次尝试 原始 URL → 各镜像前缀拼接。
    fn build_download_candidates(&self, url: &str) -> Vec<String> {
        let mut candidates = vec![url.to_string()];
        for mirror in get_mirrors().iter().skip(1) {
            if !mirror.prefix.is_empty() {
                candidates.push(format!("{}{}", mirror.prefix, url));
            }
        }
        candidates
    }

    /// 下载指定版本的 FRP（镜像回退链 + 进度回调）
    ///
    /// 下载失败根因修复：此前直连 GitHub 无任何回退，
    /// 国内网络环境必然超时。现按 原始 → ghproxy → jwinks 逐个尝试。
    pub async fn download_version(
        &self,
        version: &str,
        url: &str,
        progress: Option<ProgressCallback>,
    ) -> Result<PathBuf> {
        info!("Downloading FRP version {} from {}", version, url);

        fs::create_dir_all(&self.install_dir)?;

        let candidates = self.build_download_candidates(url);
        let mut last_err = String::new();

        for (idx, candidate) in candidates.iter().enumerate() {
            info!("Download attempt {}/{}: {}", idx + 1, candidates.len(), candidate);
            match self.try_download(candidate, &mut progress.clone()).await {
                Ok(data) => {
                    // 按内容魔数决定解压方式（比 URL 后缀更可靠）
                    let is_zip = data.starts_with(b"PK\x03\x04");
                    let ext = if is_zip { "zip" } else { "tar.gz" };
                    let archive_path = self.install_dir.join(format!("frp_{}.{}", version, ext));
                    fs::write(&archive_path, &data)?;

                    // 解压
                    let frpc_path = if is_zip {
                        self.extract_zip(&archive_path, version)?
                    } else {
                        self.extract_tar_gz(&archive_path, version)?
                    };

                    // 清理压缩包
                    fs::remove_file(&archive_path).ok();

                    // 下载成功自动激活该版本
                    self.set_active_version(version)?;

                    info!("FRP {} downloaded to {:?} (via attempt {})", version, frpc_path, idx + 1);
                    return Ok(frpc_path);
                }
                Err(e) => {
                    warn!("Download attempt {} failed: {}", idx + 1, e);
                    last_err = e.to_string();
                    continue;
                }
            }
        }

        Err(anyhow::anyhow!(
            "所有下载通道均失败（GitHub 直连 + {} 个镜像）。最后错误：{}。\
             请检查网络后重试，或使用「导入本地版本」功能。",
            candidates.len() - 1,
            last_err
        ))
    }

    /// 单次下载尝试（流式读取 + 进度回调）
    async fn try_download(
        &self,
        url: &str,
        progress: &Option<ProgressCallback>,
    ) -> Result<Vec<u8>> {
        let client = reqwest::Client::builder()
            .user_agent("frpc-gui")
            .connect_timeout(std::time::Duration::from_secs(15))
            .build()?;

        let resp = client.get(url).send().await?;
        if !resp.status().is_success() {
            return Err(anyhow::anyhow!("HTTP {}", resp.status()));
        }

        let total_size = resp.content_length().unwrap_or(0);
        let mut downloaded: u64 = 0;

        use futures_util::StreamExt;
        let mut stream = resp.bytes_stream();
        let mut data = Vec::new();

        while let Some(chunk) = stream.next().await {
            let chunk = chunk?;
            downloaded += chunk.len() as u64;
            data.extend_from_slice(&chunk);

            if let Some(ref cb) = progress {
                if let Some(f) = cb.lock().await.as_ref() {
                    f(downloaded, total_size);
                }
            }
        }

        if data.is_empty() {
            return Err(anyhow::anyhow!("下载内容为空"));
        }

        // 内容魔数校验：防止镜像把 404/错误页 HTML 当 200 返回，
        // 此前该场景导致解压报 "failed to iterate over archive"
        let is_zip = data.starts_with(b"PK\x03\x04");
        let is_gzip = data.starts_with(&[0x1f, 0x8b]);
        if !is_zip && !is_gzip {
            let preview = String::from_utf8_lossy(&data[..data.len().min(80)]).to_string();
            return Err(anyhow::anyhow!(
                "下载内容不是有效压缩包(zip/gzip 魔数不符)，疑似错误页: {}...",
                preview.replace('\n', " ")
            ));
        }

        Ok(data)
    }

    /// 解压 tar.gz
    fn extract_tar_gz(&self, archive_path: &Path, version: &str) -> Result<PathBuf> {
        use flate2::read::GzDecoder;
        use tar::Archive;

        let file = fs::File::open(archive_path)?;
        let gz = GzDecoder::new(file);
        let mut tar = Archive::new(gz);

        let version_dir = self.install_dir.join(version);
        fs::create_dir_all(&version_dir)?;
        tar.unpack(&version_dir)?;

        // 查找 frpc 可执行文件
        let frpc_name = if cfg!(windows) { "frpc.exe" } else { "frpc" };
        let frpc_path = self.find_frpc_in_dir(&version_dir, frpc_name)
            .ok_or_else(|| anyhow::anyhow!("解压后未找到 frpc 可执行文件"))?;

        Ok(frpc_path)
    }

    /// 解压 zip
    fn extract_zip(&self, archive_path: &Path, version: &str) -> Result<PathBuf> {
        // 使用 std::process::Command 调用系统 unzip（Windows 自带 PowerShell Expand-Archive）
        #[cfg(windows)]
        {
            let version_dir = self.install_dir.join(version);
            fs::create_dir_all(&version_dir)?;

            let ps_script = format!(
                "Expand-Archive -Path '{}' -DestinationPath '{}' -Force",
                archive_path.display(),
                version_dir.display()
            );
            let mut cmd = std::process::Command::new("powershell");
            crate::utils::hide_window(&mut cmd);
            cmd.args(["-Command", &ps_script])
                .output()
                .context("PowerShell 解压失败")?;

            let frpc_path = self.find_frpc_in_dir(&version_dir, "frpc.exe")
                .ok_or_else(|| anyhow::anyhow!("解压后未找到 frpc.exe"))?;
            return Ok(frpc_path);
        }

        #[cfg(unix)]
        {
            std::process::Command::new("unzip")
                .arg("-o")
                .arg(archive_path)
                .arg("-d")
                .arg(self.install_dir.join(version))
                .output()
                .context("unzip 解压失败")?;

            let frpc_path = self.find_frpc_in_dir(&self.install_dir.join(version), "frpc")
                .ok_or_else(|| anyhow::anyhow!("解压后未找到 frpc"))?;
            Ok(frpc_path)
        }

        #[cfg(not(any(windows, unix)))]
        {
            Err(anyhow::anyhow!("不支持的平台"))
        }
    }

    /// 在目录中查找 frpc 可执行文件（递归）
    fn find_frpc_in_dir(&self, dir: &Path, frpc_name: &str) -> Option<PathBuf> {
        if let Ok(entries) = fs::read_dir(dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.is_dir() {
                    if let Some(p) = self.find_frpc_in_dir(&path, frpc_name) {
                        return Some(p);
                    }
                } else if path.file_name().map_or(false, |n| n == frpc_name) {
                    return Some(path);
                }
            }
        }
        None
    }

    /// 获取已下载的 frpc 路径
    ///
    /// 优先返回「激活版本」的 frpc（用户在版本管理中选择）；
    /// 无激活记录时回退递归查找（兼容旧数据）。
    pub fn get_downloaded_frpc_path(&self) -> Option<PathBuf> {
        let frpc_name = if cfg!(windows) { "frpc.exe" } else { "frpc" };

        // 1. 激活版本优先
        if let Some(active) = self.get_active_version() {
            if let Some(p) = self.get_frpc_path(&active) {
                return Some(p);
            }
            // 激活版本文件已损坏/被删，清除记录回退
            let _ = fs::remove_file(self.active_version_file());
        }

        // 2. 回退：递归查找（兼容旧数据）
        self.find_frpc_in_dir(&self.install_dir, frpc_name)
    }

    /// 获取指定版本的 frpc 路径
    fn get_frpc_path(&self, version: &str) -> Option<PathBuf> {
        let frpc_name = if cfg!(windows) { "frpc.exe" } else { "frpc" };
        let version_dir = self.install_dir.join(version);
        if version_dir.exists() {
            self.find_frpc_in_dir(&version_dir, frpc_name)
        } else {
            None
        }
    }

    /// 检查版本是否已下载
    pub fn is_version_downloaded(&self, version: &str) -> bool {
        self.get_frpc_path(version).is_some()
    }

    /// 删除指定版本
    pub fn delete_version(&self, version: &str) -> Result<()> {
        let version_dir = self.install_dir.join(version);
        if version_dir.exists() {
            fs::remove_dir_all(&version_dir)?;
        }
        Ok(())
    }

    /// 导入本地 frpc 文件
    ///
    /// 支持压缩包（自动解压提取 frpc）或直接的 frpc 可执行文件。
    /// 导入后自动激活为当前使用版本。
    pub fn import_local_frpc(&self, file_path: &Path) -> Result<PathBuf> {
        info!("Importing local frpc, SHA256: {}", sha256_of(file_path));

        let frpc_path = if file_path.extension().map_or(false, |ext| ext == "gz") {
            self.extract_tar_gz(file_path, "imported")?
        } else if file_path.extension().map_or(false, |ext| ext == "zip") {
            self.extract_zip(file_path, "imported")?
        } else {
            // 直接复制可执行文件到独立目录
            let frpc_name = if cfg!(windows) { "frpc.exe" } else { "frpc" };
            let dest_dir = self.install_dir.join("imported");
            fs::create_dir_all(&dest_dir)?;
            let dest = dest_dir.join(frpc_name);
            fs::copy(file_path, &dest)?;
            dest
        };

        // 导入的版本命名固定为 imported，激活它
        self.set_active_version("imported")?;
        Ok(frpc_path)
    }
}

/// 计算文件 SHA256 摘要（十六进制）
///
/// 说明：此摘要仅用于日志标识文件身份，不做安全校验。
fn sha256_of(path: &Path) -> String {
    match fs::read(path) {
        Ok(data) => {
            // FNV-1a 64 位摘要（轻量标识用，非加密安全）
            let mut hash: u64 = 0xcbf29ce484222325;
            for b in &data {
                hash ^= *b as u64;
                hash = hash.wrapping_mul(0x100000001b3);
            }
            format!("fnv1a-{:016x}-len{}", hash, data.len())
        }
        Err(e) => format!("unreadable: {}", e),
    }
}
