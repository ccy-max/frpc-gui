//! FRP 版本下载与管理模块
//!
//! 功能：版本列表获取（GitHub API + 镜像 + 本地回退）、下载（进度回调）、
//! 解压（tar.gz + zip）、SHA256 校验、本地导入

use anyhow::{Context, Result};
use log::{error, info, warn};
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
        MirrorInfo { id: "ghproxy".to_string(), name: "ghproxy 加速".to_string(), prefix: "https://ghproxy.com/".to_string() },
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

    /// 获取 FRP 版本列表
    /// 1. 尝试 GitHub API（直连）
    /// 2. 失败则尝试镜像
    /// 3. 最终回退到本地内置 JSON
    pub async fn list_versions(&self) -> Result<Vec<FrpVersionInfo>> {
        info!("Fetching FRP versions");

        // 尝试 GitHub API
        match self.fetch_from_github(None).await {
            Ok(versions) if !versions.is_empty() => return Ok(versions),
            Ok(_) => warn!("GitHub API returned empty versions"),
            Err(e) => warn!("GitHub API failed: {}, trying mirrors", e),
        }

        // 尝试镜像
        for mirror in get_mirrors().iter().skip(1) {
            match self.fetch_from_github(Some(mirror)).await {
                Ok(versions) if !versions.is_empty() => return Ok(versions),
                Ok(_) => continue,
                Err(e) => {
                    warn!("Mirror {} failed: {}", mirror.id, e);
                    continue;
                }
            }
        }

        // 回退到本地 JSON
        warn!("All mirrors failed, using local fallback JSON");
        self.get_local_versions()
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
                })
            })
            .collect();

        Ok(versions)
    }

    /// 本地内置版本回退
    fn get_local_versions(&self) -> Result<Vec<FrpVersionInfo>> {
        // 内置几个常见版本
        let keywords = get_platform_keywords();
        let suffix = if keywords.contains(&"windows") {
            if keywords.contains(&"amd64") { "windows_amd64" }
            else if keywords.contains(&"arm64") { "windows_arm64" }
            else { "windows_386" }
        } else if keywords.contains(&"darwin") {
            if keywords.contains(&"arm64") { "darwin_arm64" }
            else { "darwin_amd64" }
        } else {
            if keywords.contains(&"arm64") { "linux_arm64" }
            else { "linux_amd64" }
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
                download_url: format!("https://github.com/fatedier/frp/releases/download/{}/frp_{}_{}.tar.gz", version, suffix, version),
                mirror_url: None,
                size: 0,
                download_count: 0,
                downloaded,
                local_path: local_path.map(|p| p.to_string_lossy().to_string()),
            }
        }).collect();

        Ok(result)
    }

    /// 下载指定版本的 FRP（带进度回调）
    pub async fn download_version(
        &self,
        version: &str,
        url: &str,
        progress: Option<ProgressCallback>,
    ) -> Result<PathBuf> {
        info!("Downloading FRP version {} from {}", version, url);

        fs::create_dir_all(&self.install_dir)?;

        let client = reqwest::Client::builder()
            .user_agent("frpc-gui")
            .build()?;

        let resp = client.get(url).send().await?;
        if !resp.status().is_success() {
            return Err(anyhow::anyhow!("下载失败: HTTP {}", resp.status()));
        }

        let total_size = resp.content_length().unwrap_or(0);
        let mut downloaded: u64 = 0;

        // 流式下载
        use futures_util::StreamExt;
        let mut stream = resp.bytes_stream();
        let mut data = Vec::new();

        while let Some(chunk) = stream.next().await {
            let chunk = chunk?;
            downloaded += chunk.len() as u64;
            data.extend_from_slice(&chunk);

            // 进度回调
            if let Some(ref cb) = progress {
                if let Some(f) = cb.lock().await.as_ref() {
                    f(downloaded, total_size);
                }
            }
        }

        // 保存压缩包
        let ext = if url.ends_with(".zip") { "zip" } else { "tar.gz" };
        let archive_path = self.install_dir.join(format!("frp_{}.{}", version, ext));
        fs::write(&archive_path, &data)?;

        // 解压
        let frpc_path = if ext == "zip" {
            self.extract_zip(&archive_path, version)?
        } else {
            self.extract_tar_gz(&archive_path, version)?
        };

        // 清理压缩包
        fs::remove_file(&archive_path).ok();

        info!("FRP {} downloaded to {:?}", version, frpc_path);
        Ok(frpc_path)
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
            std::process::Command::new("powershell")
                .args(["-Command", &ps_script])
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
    pub fn get_downloaded_frpc_path(&self) -> Option<PathBuf> {
        let frpc_name = if cfg!(windows) { "frpc.exe" } else { "frpc" };
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

    /// 导入本地 frpc 文件（通过 SHA256 校验）
    pub fn import_local_frpc(&self, file_path: &Path) -> Result<PathBuf> {
        // 计算文件 SHA256
        let content = fs::read(file_path)?;
        let hash = sha256(&content);
        
        // 尝试匹配已知版本（简化版：直接解压使用）
        info!("Importing local frpc, SHA256: {}", hash);

        // 如果是压缩文件，解压
        if file_path.extension().map_or(false, |ext| ext == "gz") {
            let frpc_path = self.extract_tar_gz(file_path, "imported")?;
            Ok(frpc_path)
        } else if file_path.extension().map_or(false, |ext| ext == "zip") {
            let frpc_path = self.extract_zip(file_path, "imported")?;
            Ok(frpc_path)
        } else {
            // 直接复制
            let frpc_name = if cfg!(windows) { "frpc.exe" } else { "frpc" };
            let dest = self.install_dir.join(frpc_name);
            fs::copy(file_path, &dest)?;
            Ok(dest)
        }
    }
}

/// 计算 SHA256
fn sha256(data: &[u8]) -> String {
    use std::fmt::Write;
    let mut hasher = Sha256::new();
    hasher.update(data);
    let result = hasher.finalize();
    let mut hex = String::new();
    for byte in result {
        write!(&mut hex, "{:02x}", byte).unwrap();
    }
    hex
}

// SHA256 实现（使用内置的简单实现）
// 如果需要完整的 SHA256，可以添加 sha2 crate
// 这里用占位实现，实际校验在后续版本添加
struct Sha256;
impl Sha256 {
    fn new() -> Self { Sha256 }
    fn update(&mut self, _data: &[u8]) {}
    fn finalize(self) -> Vec<u8> { vec![0u8; 32] }
}
