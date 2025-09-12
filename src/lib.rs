//! CLI音乐播放器核心库
//!
//! 包含播放器核心功能实现，可被其他项目复用

pub use anyhow::{Result as AnyResult, anyhow};
pub use player::Player;
mod player;
