//! FRP 核心模块

pub mod config;
pub mod download;
pub mod process;

pub use config::{ConfigManager, FrpConfig, validate_config};
pub use download::{FrpVersionManager, FrpVersionInfo};
pub use process::{FrpProcessManager, ProcessState, check_frpc_exists, get_frpc_version};
