//! FRP 版本下载与管理模块

use anyhow::{Context, Result};
use log::{error, info};
use serde::{Deserialize, Serialize};
use std::fs;
use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use flate2::read::GzDecoder;
use tar::Archive;

/// GitHub Release 信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GitHubRelease {
    pub tag_name: String,
    pub name: String,
    pub published_at: String,
    pub assets: Vec<GitHubAsset>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GitHubAsset {
    pub name: String,
    pub download_url: String,
    pub size: u64,
}

/// FRP 版本信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FrpVersionInfo {
    pub version: String,
    pub name: String,
    pub published_at: String,
    pub download_url: String,
    pub size: u64,
    pub downloaded: bool,
}

/// FRP 版本管理器
pub struct FrpVersionManager {
    install_dir: PathBuf,
}

impl FrpVersionManager {
    pub fn new(install_dir: PathBuf) -> Self {
        Self { install_dir }
    }

    /// 从 GitHub API 获取 FRP 版本列表
    pub async fn list_versions(&self) -> Result<Vec<FrpVersionInfo>> {
        info!("Fetching FRP versions from GitHub API");

        let client = reqwest::Client::builder()
            .user_agent("frpc-gui")
            .build()?;

        let resp = client
            .get("https://api.github.com/repos/fatedier/frp/releases")
            .send()
            .await
            .with_context(|| "请求 GitHub API 失败")?;

        let releases: Vec<GitHubRelease> = resp
            .json()
            .await
            .with_context(|| "解析 GitHub API 响应失败")?;

        // 确定当前平台对应的资源名后缀
        let platform_suffix = self.get_platform_suffix();

        let versions: Vec<FrpVersionInfo> = releases
            .iter()
            .filter_map(|release| {
                // 找到匹配当前平台的资源
                let asset = release
                    .assets
                    .iter()
                    .find(|a| a.name.contains(platform_suffix))?;

                let version = release.tag_name.clone();
                let downloaded = self.is_version_downloaded(&version);

                Some(FrpVersionInfo {
                    version,
                    name: release.name.clone(),
                    published_at: release.published_at.clone(),
                    download_url: asset.download_url.clone(),
                    size: asset.size,
                    downloaded,
                })
            })
            .collect();

        Ok(versions)
    }

    /// 下载指定版本的 FRP
    pub async fn download_version(&self, version: &str, url: &str) -> Result<PathBuf> {
        info!("Downloading FRP version {} from {}", version, url);

        // 确保安装目录存在
        fs::create_dir_all(&self.install_dir)?;

        let client = reqwest::Client::builder()
            .user_agent("frpc-gui")
            .build()?;

        // 下载文件
        let resp = client.get(url).send().await?;
        if !resp.status().is_success() {
            return Err(anyhow::anyhow!("下载失败: HTTP {}", resp.status()));
        }

        let body = resp.bytes().await?;
        let archive_path = self.install_dir.join(format!("frp_{}.tar.gz", version));
        fs::write(&archive_path, &body)?;

        // 解压
        let frpc_path = self.extract_archive(&archive_path, version)?;
        
        // 清理压缩包
        fs::remove_file(&archive_path).ok();

        info!("FRP {} downloaded to {:?}", version, frpc_path);
        Ok(frpc_path)
    }

    /// 解压 tar.gz 归档
    fn extract_archive(&self, archive_path: &Path, version: &str) -> Result<PathBuf> {
        let file = fs::File::open(archive_path)?;
        let gz = GzDecoder::new(file);
        let mut tar = Archive::new(gz);

        // 解压到安装目录
        tar.unpack(&self.install_dir)?;

        // 找到 frpc 可执行文件
        // frp 归档结构: frp_xxx_version/frpc
        let dir_name = format!("frp_{}_{}", self.get_platform_suffix(), version);
        let frpc_name = if cfg!(windows) { "frpc.exe" } else { "frpc" };
        let frpc_path = self.install_dir.join(&dir_name).join(frpc_name);

        if frpc_path.exists() {
            Ok(frpc_path)
        } else {
            // 尝试直接在安装目录找
            let alt_path = self.install_dir.join(frpc_name);
            if alt_path.exists() {
                Ok(alt_path)
            } else {
                Err(anyhow::anyhow!("解压后未找到 frpc 可执行文件"))
            }
        }
    }

    /// 检查版本是否已下载
    pub fn is_version_downloaded(&self, version: &str) -> bool {
        let platform_suffix = self.get_platform_suffix();
        let dir_name = format!("frp_{}_{}", platform_suffix, version);
        let frpc_name = if cfg!(windows) { "frpc.exe" } else { "frpc" };
        let frpc_path = self.install_dir.join(&dir_name).join(frpc_name);
        
        if frpc_path.exists() {
            return true;
        }
        // 也检查直接路径
        self.install_dir.join(frpc_name).exists()
    }

    /// 获取已下载的 frpc 路径
    pub fn get_downloaded_frpc_path(&self) -> Option<PathBuf> {
        let frpc_name = if cfg!(windows) { "frpc.exe" } else { "frpc" };
        
        // 搜索所有子目录
        if let Ok(entries) = fs::read_dir(&self.install_dir) {
            for entry in entries.flatten() {
                let path = entry.path().join(frpc_name);
                if path.exists() {
                    return Some(path);
                }
            }
        }
        
        // 检查根目录
        let direct_path = self.install_dir.join(frpc_name);
        if direct_path.exists() {
            return Some(direct_path);
        }
        
        None
    }

    /// 删除指定版本
    pub fn delete_version(&self, version: &str) -> Result<()> {
        let platform_suffix = self.get_platform_suffix();
        let dir_name = format!("frp_{}_{}", platform_suffix, version);
        let version_dir = self.install_dir.join(&dir_name);
        
        if version_dir.exists() {
            fs::remove_dir_all(&version_dir)?;
        }
        
        Ok(())
    }

    /// 获取当前平台对应的文件名后缀
    fn get_platform_suffix(&self) -> &'static str {
        let os = std::env::consts::OS;
        let arch = std::env::consts::ARCH;
        
        match (os, arch) {
            ("windows", "x86_64") => "windows_amd64",
            ("windows", "x86") => "windows_386",
            ("linux", "x86_64") => "linux_amd64",
            ("linux", "aarch64") => "linux_arm64",
            ("macos", "x86_64") => "darwin_amd64",
            ("macos", "aarch64") => "darwin_arm64",
            _ => "linux_amd64",
        }
    }
}
