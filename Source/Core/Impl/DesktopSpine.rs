//! # DesktopSpine.rs
//! 
//! Concrete implementation of the Universal Spine traits for the Desktop target.
//! Uses `std::fs` for file I/O and `tauri` for window management.

use async_trait::async_trait;
use std::fs;
use std::path::PathBuf;
use tauri::Manager;
use crate::Core::Spine::{FileSystemSpine, WindowManagerSpine, LifecycleSpine, DialogOptions, ClientInfo, ServerInfo};

// --- Filesystem Implementation ---

#[derive(Clone)]
pub struct DesktopFileSystem;

#[async_trait]
impl FileSystemSpine for DesktopFileSystem {
    async fn read_file(&self, path: PathBuf) -> Result<Vec<u8>, String> {
        // Run blocking I/O on a thread pool to avoid blocking async runtime
        tokio::task::spawn_blocking(move || {
            fs::read(&path).map_err(|e| format!("Failed to read {}: {}", path.display(), e))
        })
        .await
        .map_err(|e| e.to_string())?
    }

    async fn write_file(&self, path: PathBuf, content: Vec<u8>) -> Result<(), String> {
        tokio::task::spawn_blocking(move || {
            fs::write(&path, content).map_err(|e| format!("Failed to write {}: {}", path.display(), e))
        })
        .await
        .map_err(|e| e.to_string())?
    }

    async fn list_dir(&self, path: PathBuf) -> Result<Vec<String>, String> {
        tokio::task::spawn_blocking(move || {
            let mut entries = Vec::new();
            if path.is_dir() {
                for entry in fs::read_dir(&path).map_err(|e| e.to_string())? {
                    let entry = entry.map_err(|e| e.to_string())?;
                    if let Ok(name) = entry.file_name().into_string() {
                        entries.push(name);
                    }
                }
            }
            Ok(entries)
        })
        .await
        .map_err(|e| e.to_string())?
    }

    async fn exists(&self, path: PathBuf) -> bool {
        path.exists()
    }
}

// --- Window Manager Implementation ---

#[derive(Clone)]
pub struct TauriWindowManager {
    pub app_handle: tauri::AppHandle,
}

#[async_trait]
impl WindowManagerSpine for TauriWindowManager {
    async fn show_message(&self, title: &str, message: &str, level: &str) {
        let app = self.app_handle.clone();
        let title = title.to_string();
        let msg = message.to_string();
        let _level = level.to_string(); // Logic to map 'info'/'error' to icon can go here

        // Dispatch to main thread
        tauri::async_runtime::spawn(async move {
            if let Some(window) = app.get_window("main") {
                tauri::api::dialog::message(Some(&window), &title, &msg);
            } else {
                eprintln!("[Headless] Message: {} - {}", title, msg);
            }
        });
    }

    async fn open_dialog(&self, _options: DialogOptions) -> Option<PathBuf> {
        // Implement native file picker
        // For now, return None as placeholder
        None
    }
}

// --- Lifecycle Implementation ---

#[derive(Clone)]
pub struct DesktopLifecycle;

#[async_trait]
impl LifecycleSpine for DesktopLifecycle {
    async fn handshake(&self, client_info: ClientInfo) -> Result<ServerInfo, String> {
        println!("[DesktopLifecycle] Handshake received from role: {}, pid: {}", client_info.role, client_info.pid);
        
        Ok(ServerInfo {
            version: "0.1.0".to_string(),
            capabilities: vec![
                "fs.read".to_string(),
                "fs.write".to_string(),
                "window.dialog".to_string(),
                "system.lifecycle".to_string(),
            ],
        })
    }

    async fn shutdown(&self) {
        println!("[DesktopLifecycle] Shutdown requested");
        std::process::exit(0);
    }
}
