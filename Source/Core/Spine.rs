//! # Spine.rs
//! 
//! The Universal Spine defines the abstract traits that drive the Mountain editor.
//! These traits decouple the "what" (Business Logic) from the "how" (Implementation/Transport).
//! 
//! By coding against these traits, Mountain can support multiple targets:
//! - Desktop (Tauri + Local FS)
//! - Web (DOM + OPFS/Memory FS)
//! - CLI (Terminal + Stdio)

use async_trait::async_trait;
use std::path::PathBuf;
use serde::{Deserialize, Serialize};

/// v0.1: The Filesystem Spine
/// Abstracting disk I/O so it can be swapped (e.g., In-Memory for Web)
#[async_trait]
pub trait FileSystemSpine: Send + Sync {
    async fn read_file(&self, path: PathBuf) -> Result<Vec<u8>, String>;
    async fn write_file(&self, path: PathBuf, content: Vec<u8>) -> Result<(), String>;
    async fn list_dir(&self, path: PathBuf) -> Result<Vec<String>, String>;
    async fn exists(&self, path: PathBuf) -> bool;
}

/// v0.2: The Window Manager Spine
/// Abstracting UI so we can swap Tauri for DOM (Web) or Terminal (CLI)
#[async_trait]
pub trait WindowManagerSpine: Send + Sync {
    async fn show_message(&self, title: &str, message: &str, level: &str);
    async fn open_dialog(&self, options: DialogOptions) -> Option<PathBuf>;
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DialogOptions {
    pub title: Option<String>,
    pub filters: Vec<String>,
}

/// v0.3: Lifecycle Spine
#[async_trait]
pub trait LifecycleSpine: Send + Sync {
    async fn handshake(&self, client_info: ClientInfo) -> Result<ServerInfo, String>;
    async fn shutdown(&self);
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClientInfo {
    pub pid: u32,
    pub role: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerInfo {
    pub version: String,
    pub capabilities: Vec<String>,
}
