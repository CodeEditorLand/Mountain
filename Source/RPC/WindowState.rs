//! # Window State Management
//!
//! This module provides state management for window-related operations,
//! including status bar items and webview panels.

use std::{collections::HashMap, sync::Arc};

use parking_lot::RwLock;
use serde::{Deserialize, Serialize};

/// Status bar item metadata
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct StatusBarItem {
	/// Unique identifier for the status bar item
	pub id:String,

	/// Text to display
	pub text:String,

	/// Tooltip text (optional)
	pub tooltip:String,

	/// Whether the item is visible
	pub visible:bool,

	/// Priority for determining position
	pub priority:i32,
}

impl StatusBarItem {
	/// Creates a new status bar item
	pub fn new(id:String, text:String, tooltip:String) -> Self { Self { id, text, tooltip, visible:true, priority:0 } }
}

/// Webview panel metadata
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct WebviewPanel {
	/// Unique handle for the webview panel
	pub handle:u32,

	/// View type identifier
	pub view_type:String,

	/// Title of the panel
	pub title:String,

	/// Optional icon path
	pub icon_path:Option<String>,

	/// View column where the panel is displayed
	pub view_column:i32,

	/// Whether the panel is visible
	pub visible:bool,
}

impl WebviewPanel {
	/// Creates a new webview panel
	pub fn new(handle:u32, view_type:String, title:String, icon_path:Option<String>, view_column:i32) -> Self {
		Self { handle, view_type, title, icon_path, view_column, visible:true }
	}
}

/// Window state manager
///
/// This singleton manages the state for all window-related operations:
/// - Status bar items registry
/// - Webview panels registry
/// - Next handle generation
#[derive(Clone)]
pub struct WindowStateManager {
	/// Registry of status bar items
	status_items:Arc<RwLock<HashMap<String, StatusBarItem>>>,

	/// Registry of webview panels (handle -> panel)
	webview_panels:Arc<RwLock<HashMap<u32, WebviewPanel>>>,

	/// Next handle to assign for webview panels
	next_handle:Arc<RwLock<u32>>,
}

impl WindowStateManager {
	/// Create a new window state manager
	pub fn new() -> Self {
		Self {
			status_items:Arc::new(RwLock::new(HashMap::new())),
			webview_panels:Arc::new(RwLock::new(HashMap::new())),
			next_handle:Arc::new(RwLock::new(1)),
		}
	}

	// ==================== Status Bar Operations ====================

	/// Register a status bar item
	pub fn register_status_item(&self, item:StatusBarItem) {
		let mut items = self.status_items.write();
		items.insert(item.id.clone(), item);
	}

	/// Get a status bar item by ID
	pub fn get_status_item(&self, id:&str) -> Option<StatusBarItem> {
		let items = self.status_items.read();
		items.get(id).cloned()
	}

	/// Update status bar item text
	pub fn update_status_item_text(&self, id:&str, text:&str) -> bool {
		let mut items = self.status_items.write();
		if let Some(item) = items.get_mut(id) {
			item.text = text.to_string();
			return true;
		}
		false
	}

	/// Remove a status bar item
	pub fn remove_status_item(&self, id:&str) -> bool {
		let mut items = self.status_items.write();
		items.remove(id).is_some()
	}

	/// Get all status bar items
	pub fn get_all_status_items(&self) -> Vec<StatusBarItem> {
		let items = self.status_items.read();
		items.values().cloned().collect()
	}

	// ==================== Webview Operations ====================

	/// Generate next webview handle
	pub fn next_webview_handle(&self) -> u32 {
		let mut handle = self.next_handle.write();
		let current = *handle;
		*handle = handle.wrapping_add(1);
		current
	}

	/// Register a webview panel
	pub fn register_webview_panel(&self, panel:WebviewPanel) {
		let mut panels = self.webview_panels.write();
		panels.insert(panel.handle, panel);
	}

	/// Get a webview panel by handle
	pub fn get_webview_panel(&self, handle:u32) -> Option<WebviewPanel> {
		let panels = self.webview_panels.read();
		panels.get(&handle).cloned()
	}

	/// Remove a webview panel
	pub fn remove_webview_panel(&self, handle:u32) -> bool {
		let mut panels = self.webview_panels.write();
		panels.remove(&handle).is_some()
	}

	/// Get all webview panels
	pub fn get_all_webview_panels(&self) -> Vec<WebviewPanel> {
		let panels = self.webview_panels.read();
		panels.values().cloned().collect()
	}
}

impl Default for WindowStateManager {
	fn default() -> Self { Self::new() }
}
