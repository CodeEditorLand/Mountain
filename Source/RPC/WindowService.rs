//! # WindowService Implementation
//!
//! This module implements window-related gRPC service methods for the
//! Mountain backend. These methods handle UI operations that need to be
//! delegated to the Wind frontend via IPC.
//!
//! ## Service Responsibilities
//!
//! - **Documents**: Opening and managing text documents
//! - **Messages**: Displaying information, warning, and error messages
//! - **Status Bar**: Creating and updating status bar items
//! - **Webview Panels**: Creating and managing webview panels
//!
//! ## Architecture
//!
//! The WindowService maintains references to:
//! - `MountainEnvironment`: Access to all Mountain services
//! - IPC transport for communicating with Wind
//!
//! ## Implementation Notes
//!
//! This service is a subset of the main CocoonService, focusing specifically
//! on window and UI operations. Most of these operations will be delegated to
//! Wind via the IPC layer, as Wind controls the actual UI/window state.

use std::sync::Arc;

use async_trait::async_trait;
use log::{debug, error, info, warn};
use tonic::{Request, Response, Status};

use crate::Environment::MountainEnvironment::MountainEnvironment;
use CommonLibrary::Environment::Requires::Requires;

// Import generated protobuf types
use crate::Vine::Generated::{
	// Common types
	Empty,
	Uri,
	ViewColumn,

	// Window Operations
	ShowTextDocumentRequest,
	ShowTextDocumentResponse,
	ShowMessageRequest,
	ShowMessageResponse,
	CreateStatusBarItemRequest,
	CreateStatusBarItemResponse,
	SetStatusBarTextRequest,
	CreateWebviewPanelRequest,
	CreateWebviewPanelResponse,
	SetWebviewHtmlRequest,
	OnDidReceiveMessageRequest,
};

// Import state management
use super::WindowState::{StatusBarItem, WindowStateManager, WebviewPanel};

/// WindowService handles window and UI-related operations
///
/// This service manages interactions with the Wind frontend for:
/// - Opening text documents
/// - Displaying messages to the user
/// - Managing status bar items
/// - Creating and managing webview panels
#[derive(Clone)]
pub struct WindowService {
	/// Mountain environment providing access to all services
	environment: Arc<MountainEnvironment>,

	/// Window state manager for status bars and webviews
	state_manager: Arc<WindowStateManager>,
}

impl WindowService {
	/// Creates a new instance of the WindowService
	///
	/// # Parameters
	/// - `environment`: Mountain environment with access to all services
	///
	/// # Returns
	/// A new WindowService instance
	pub fn new(environment: Arc<MountainEnvironment>) -> Self {
		info!("[WindowService] New instance created");

		Self {
			environment,
			state_manager: Arc::new(WindowStateManager::new()),
		}
	}
}

impl WindowService {
	// ==================== Document Operations ====================

	/// Show a text document in the editor
	///
	/// This method instructs Wind to open a text document at the specified URI.
	///
	/// # Parameters
	/// - `uri`: The URI of the document to open
	/// - `view_column`: The view column to use (optional)
	/// - `preserve_focus`: Whether to preserve the current focus (optional)
	///
	/// # Returns
	/// Success status indicating whether the document was opened
	pub async fn show_text_document_impl(
	&self,
	uri: &Uri,
	view_column: Option<ViewColumn>,
	preserve_focus: Option<bool>,
) -> Result<bool, Status> {
	let uri_value = &uri.value;
	info!(
		"[WindowService] Showing text document: {} (column: {:?}, preserve_focus: {:?})",
		uri_value, view_column, preserve_focus
	);

	// Use DocumentProvider from MountainEnvironment to open document
	let document_provider = self.environment.Require();
	match document_provider.OpenDocument(uri_value.to_string()).await {
		Ok(_) => {
			info!("[WindowService] Document opened successfully: {}", uri_value);
			Ok(true)
		},
		Err(error) => {
			error!(
				"[WindowService] Failed to open document {}: {}",
				uri_value, error
			);
			Err(Status::internal(format!("Failed to open document: {}", error)))
		},
	}
}

	// ==================== Message Operations ====================

	/// Show an information message to the user
	///
	/// # Parameters
	/// - `message`: The message text to display
	///
	/// # Returns
	/// Success status
	pub async fn show_information_message_impl(
	&self,
	message: &str,
) -> Result<bool, Status> {
	debug!("[WindowService] Showing information message: {}", message);

	// Use UserInterfaceProvider from MountainEnvironment
	let ui_provider = self.environment.Require();
	match ui_provider.ShowInformationMessage(message.to_string()).await {
		Ok(_) => {
			info!("[WindowService] Information message shown");
			Ok(true)
		},
		Err(error) => {
			error!("[WindowService] Failed to show information message: {}", error);
			warn!("{}", message); // Fallback to logging
			Ok(true) // Consider non-blocking errors as success
		},
	}
}

	/// Show a warning message to the user
	///
	/// # Parameters
	/// - `message`: The message text to display
	///
	/// # Returns
	/// Success status
	pub async fn show_warning_message_impl(&self, message: &str) -> Result<bool, Status> {
		debug!("[WindowService] Showing warning message: {}", message);

		// Use UserInterfaceProvider from MountainEnvironment
		let ui_provider = self.environment.Require();
		match ui_provider.ShowWarningMessage(message.to_string()).await {
			Ok(_) => {
				info!("[WindowService] Warning message shown");
				Ok(true)
			},
			Err(error) => {
				error!("[WindowService] Failed to show warning message: {}", error);
				warn!("{}", message); // Fallback to logging
				Ok(true) // Consider non-blocking errors as success
			},
		}
	}

	/// Show an error message to the user
	///
	/// # Parameters
	/// - `message`: The message text to display
	///
	/// # Returns
	/// Success status
	pub async fn show_error_message_impl(&self, message: &str) -> Result<bool, Status> {
		debug!("[WindowService] Showing error message: {}", message);

		// Use UserInterfaceProvider from MountainEnvironment
		let ui_provider = self.environment.Require();
		match ui_provider.ShowErrorMessage(message.to_string()).await {
			Ok(_) => {
				info!("[WindowService] Error message shown");
				Ok(true)
			},
			Err(error) => {
				error!("[WindowService] Failed to show error message: {}", error);
				error!("{}", message); // Fallback to logging
				Ok(true) // Consider non-blocking errors as success
			},
		}
	}

	// ==================== Status Bar Operations ====================

	/// Create a status bar item
	///
	/// # Parameters
	/// - `id`: Unique identifier for the status bar item
	/// - `text`: The text to display
	/// - `tooltip`: Optional tooltip text
	///
	/// # Returns
	/// The ID of the created status bar item (same as input ID)
	pub async fn create_status_bar_item_impl(
		&self,
		id: &str,
		text: &str,
		tooltip: &str,
	) -> Result<String, Status> {
		info!(
			"[WindowService] Creating status bar item: {} (text: {}, tooltip: {})",
			id, text, tooltip
		);

		// Use StatusBarProvider from MountainEnvironment
		let status_bar_provider = self.environment.Require();

		// Register with StatusBarProvider
		match status_bar_provider.CreateStatusBarItem(id.to_string(), text.to_string(), tooltip.to_string()).await {
			Ok(_) => {
				info!("[WindowService] Status bar item created: {}", id);
				Ok(id.to_string())
			},
			Err(error) => {
				error!("[WindowService] Failed to create status bar item: {}", error);
				Err(Status::internal(format!("Failed to create status bar item: {}", error)))
			},
		}
	}

	/// Set the text of a status bar item
	///
	/// # Parameters
	/// - `item_id`: The ID of the status bar item
	/// - `text`: The new text to display
	///
	/// # Returns
	/// Success status
	pub async fn set_status_bar_text_impl(
		&self,
		item_id: &str,
		text: &str,
	) -> Result<(), Status> {
		debug!(
			"[WindowService] Setting status bar text for item {}: {}",
			item_id, text
		);

		// Use StatusBarProvider from MountainEnvironment
		let status_bar_provider = self.environment.Require();

		match status_bar_provider.SetStatusBarText(item_id.to_string(), text.to_string()).await {
			Ok(_) => {
				debug!("[WindowService] Status bar text updated for item: {}", item_id);
				Ok(())
			},
			Err(error) => {
				error!("[WindowService] Failed to set status bar text: {}", error);
				Err(Status::internal(format!("Failed to set status bar text: {}", error)))
			},
		}
	}

	// ==================== Webview Operations ====================

	/// Create a webview panel
	///
	/// # Parameters
	/// - `view_type`: The type of webview (e.g., 'markdown.preview')
	/// - `title`: Title of the panel
	/// - `icon_path`: Optional path to icon
	/// - `view_column`: The view column to use
	/// - `preserve_focus`: Whether to preserve current focus
	/// - `enable_find_widget`: Enable find widget
	/// - `retain_context_when_hidden`: Retain DOM context when hidden
	/// - `local_resource_roots`: Local resources allowed
	///
	/// # Returns
	/// The handle of the created webview panel
	pub async fn create_webview_panel_impl(
		&self,
		view_type: &str,
		title: &str,
		icon_path: &str,
		view_column: ViewColumn,
		preserve_focus: bool,
		enable_find_widget: bool,
		retain_context_when_hidden: bool,
		local_resource_roots: &[String],
	) -> Result<u32, Status> {
		info!(
			"[WindowService] Creating webview panel: {} (title: {})",
			view_type, title
		);

		// Use WebviewProvider from MountainEnvironment
		let webview_provider = self.environment.Require();

		// Convert ViewColumn enum to integer
		let view_column_int = view_column as i32;

		// Generate unique handle
		let handle = self.state_manager.next_webview_handle();

		match webview_provider
			.CreateWebviewPanel(
				handle,
				view_type.to_string(),
				title.to_string(),
				if icon_path.is_empty() {
					None
				} else {
					Some(icon_path.to_string())
				},
				view_column_int,
				preserve_focus,
				enable_find_widget,
				retain_context_when_hidden,
				local_resource_roots.to_vec(),
			)
			.await
		{
			Ok(_) => {
				info!("[WindowService] Webview panel created with handle: {}", handle);
				Ok(handle)
			},
			Err(error) => {
				error!("[WindowService] Failed to create webview panel: {}", error);
				Err(Status::internal(format!("Failed to create webview panel: {}", error)))
			},
		}
	}

	/// Set the HTML content of a webview panel
	///
	/// # Parameters
	/// - `handle`: The handle of the webview panel
	/// - `html`: The HTML content to set
	///
	/// # Returns
	/// Success status
	pub async fn set_webview_html_impl(&self, handle: u32, html: &str) -> Result<(), Status> {
		debug!(
			"[WindowService] Setting webview HTML for handle {}: {} characters",
			handle,
			html.len()
		);

		// Use WebviewProvider from MountainEnvironment
		let webview_provider = self.environment.Require();

		match webview_provider.SetWebviewHtml(handle, html.to_string()).await {
			Ok(_) => {
				debug!("[WindowService] Webview HTML updated for handle: {}", handle);
				Ok(())
			},
			Err(error) => {
				error!("[WindowService] Failed to set webview HTML: {}", error);
				Err(Status::internal(format!("Failed to set webview HTML: {}", error)))
			},
		}
	}

	/// Handle a message received from a webview
	///
	/// This is called when a webview sends a message back to Mountain.
	///
	/// # Parameters
	/// - `handle`: The handle of the webview panel
	/// - `message`: The message (string or bytes)
	///
	/// # Returns
	/// Success status
	pub async fn on_did_receive_message_impl(
		&self,
		handle: u32,
		message: &str,
	) -> Result<(), Status> {
		debug!(
			"[WindowService] Received webview message from handle {}: {}",
			handle, message
		);

		// TODO: Forward message to appropriate extension handler
		// - Look up extension that created the webview
		// - Send message via gRPC to extension

		Ok(())
	}
}

#[cfg(test)]
mod tests {
	use super::*;

	// TODO: Add unit tests for WindowService methods
	// These tests should mock the IPC layer to verify correct message formatting
}
