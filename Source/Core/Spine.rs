//! # Spine.rs
//! 
//! The Universal Spine defines the abstract traits that drive the Mountain editor.

use async_trait::async_trait;
use std::path::PathBuf;
use serde::{Deserialize, Serialize};
use serde_json::Value;

/// v0.1: The Filesystem Spine
#[async_trait]
pub trait FileSystemSpine: Send + Sync {
    async fn read_file(&self, path: PathBuf) -> Result<Vec<u8>, String>;
    async fn write_file(&self, path: PathBuf, content: Vec<u8>) -> Result<(), String>;
    async fn list_dir(&self, path: PathBuf) -> Result<Vec<String>, String>;
    async fn exists(&self, path: PathBuf) -> bool;
    // Add new methods for full FS support
    async fn create_dir(&self, path: PathBuf) -> Result<(), String>;
    async fn delete(&self, path: PathBuf) -> Result<(), String>;
    async fn rename(&self, from: PathBuf, to: PathBuf) -> Result<(), String>;
}

/// v0.2: The Window Manager Spine
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

/// v0.4: Configuration Spine
#[async_trait]
pub trait ConfigSpine: Send + Sync {
    async fn get(&self, section: String) -> Result<Value, String>;
    async fn set(&self, key: String, value: Value, scope: ConfigScope) -> Result<(), String>;
    async fn reload(&self) -> Result<Value, String>;
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum ConfigScope {
    Application = 0,
    Workspace = 1,
    Profile = 2,
}

/// v0.5: Terminal Spine
#[async_trait]
pub trait TerminalSpine: Send + Sync {
    async fn create(&self, options: TerminalOptions) -> Result<u32, String>;
    async fn write(&self, id: u32, data: String) -> Result<(), String>;
    async fn resize(&self, id: u32, cols: u16, rows: u16) -> Result<(), String>;
    async fn kill(&self, id: u32) -> Result<(), String>;
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TerminalOptions {
    pub name: String,
    pub shell_path: Option<String>,
    pub cwd: Option<String>,
}
