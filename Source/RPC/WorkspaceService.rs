//! # WorkspaceService Implementation
//!
//! This module implements workspace-related gRPC service methods for the
//! Mountain backend. These methods handle file operations, text search, and
//! document management.
//!
//! ## Service Responsibilities
//!
//! - **File Search**: Find files matching patterns
//! - **Text Search**: Find text across multiple files
//! - **Document Operations**: Open, save, and edit documents
//! - **Configuration**: Manage workspace configuration
//! - **Workspace Folders**: Manage workspace folder structure
//!
//! ## Architecture
//!
//! The WorkspaceService maintains references to:
//! - `MountainEnvironment`: Access to all Mountain services
//! - FileSystem provider for file operations
//! - Search provider for text operations
//!
//! ## Implementation Notes
//!
//! This service is a subset of the main CocoonService, focusing specifically
//! on workspace operations. Most document operations will be delegated to
//! Wind via the IPC layer, while file system operations use the Mountain
//! FileSystem provider.

use std::sync::Arc;

use async_trait::async_trait;
use log::{debug, error, info};
use tonic::{Request, Response, Status};

use crate::Environment::MountainEnvironment::MountainEnvironment;
use CommonLibrary::Environment::Requires::Requires;

// Import generated protobuf types
use crate::Vine::Generated::{
	// Common types
	Empty,
	Uri,
	ViewColumn,
	Range,
	TextEdit,

	// Workspace Operations
	FindFilesRequest,
	FindFilesResponse,
	FindTextInFilesRequest,
	FindTextInFilesResponse,
	TextMatch,
	OpenDocumentRequest,
	OpenDocumentResponse,
	SaveAllRequest,
	SaveAllResponse,
	ApplyEditRequest,
	ApplyEditResponse,
	UpdateConfigurationRequest,
	UpdateWorkspaceFoldersRequest,
	WorkspaceFolder,
};

/// WorkspaceService handles workspace-related operations
///
/// This service manages:
/// - File and text search operations
/// - Document opening and editing
/// - Configuration management
/// - Workspace folder management
#[derive(Clone)]
pub struct WorkspaceService {
	/// Mountain environment providing access to all services
	environment: Arc<MountainEnvironment>,
}

impl WorkspaceService {
	/// Creates a new instance of the WorkspaceService
	///
	/// # Parameters
	/// - `environment`: Mountain environment with access to all services
	///
	/// # Returns
	/// A new WorkspaceService instance
	pub fn new(environment: Arc<MountainEnvironment>) -> Self {
		info!("[WorkspaceService] New instance created");

		Self { environment }
	}
}

impl WorkspaceService {
	// ==================== Search Operations ====================

	/// Find files matching a pattern
	///
	/// # Parameters
	/// - `pattern`: The glob pattern to match (e.g., `**/*.rs`)
	/// - `include`: Whether to include or exclude matching files
	///
	/// # Returns
	/// List of matching file URIs
	pub async fn find_files_impl(
		&self,
		pattern: &str,
		include: bool,
	) -> Result<Vec<String>, Status> {
		debug!("[WorkspaceService] Finding files with pattern: {} (include: {})", pattern, include);

		// Use SearchProvider from MountainEnvironment
		let search_provider = self.environment.Require();

		match search_provider.FindFiles(pattern.to_string(), include).await {
			Ok(files) => {
				info!("[WorkspaceService] Found {} files matching pattern: {}", files.len(), pattern);
				Ok(files)
			},
			Err(err) => {
				error!("[WorkspaceService] Failed to find files: {}", err);
				Err(Status::internal(format!("Failed to find files: {}", err)))
			},
		}
	}

	/// Find text across multiple files
	///
	/// # Parameters
	/// - `pattern`: The text pattern to search for (supports regex)
	/// - `include`: File patterns to include
	/// - `exclude`: File patterns to exclude
	///
	/// # Returns
	/// List of text matches with location and preview
	pub async fn find_text_in_files_impl(
		&self,
		pattern: &str,
		include: &[String],
		exclude: &[String],
	) -> Result<Vec<TextMatch>, Status> {
		debug!(
			"[WorkspaceService] Finding text with pattern: {} (include: {:?}, exclude: {:?})",
			pattern, include, exclude
		);

		// Use SearchProvider from MountainEnvironment
		let search_provider = self.environment.Require();

		match search_provider
			.FindTextInFiles(pattern.to_string(), include.to_vec(), exclude.to_vec())
			.await
		{
			Ok(matches) => {
				info!("[WorkspaceService] Found {} text matches", matches.len());
				Ok(matches)
			},
			Err(err) => {
				error!("[WorkspaceService] Failed to search text: {}", err);
				Err(Status::internal(format!("Failed to search text: {}", err)))
			},
		}
	}

	// ==================== Document Operations ====================

	/// Open a document in the editor
	///
	/// # Parameters
	/// - `uri`: The URI of the document to open
	/// - `view_column`: The view column to use (optional)
	///
	/// # Returns
	/// Success status indicating whether the document was opened
	pub async fn open_document_impl(
		&self,
		uri: &Uri,
		view_column: Option<ViewColumn>,
	) -> Result<bool, Status> {
		let uri_value = &uri.value;
		info!(
			"[WorkspaceService] Opening document: {} (column: {:?})",
			uri_value, view_column
		);

		// Use DocumentProvider from MountainEnvironment
		let document_provider = self.environment.Require();

		match document_provider.OpenDocument(uri_value.to_string()).await {
			Ok(_) => {
				info!("[WorkspaceService] Document opened successfully: {}", uri_value);
				Ok(true)
			},
			Err(err) => {
				error!("[WorkspaceService] Failed to open document {}: {}", uri_value, err);
				Err(Status::internal(format!("Failed to open document: {}", err)))
			},
		}
	}

	/// Save all open documents
	///
	/// # Parameters
	/// - `include_untitled`: Whether to include untitled documents
	///
	/// # Returns
	/// Success status indicating whether all documents were saved
	pub async fn save_all_impl(&self, include_untitled: bool) -> Result<bool, Status> {
		info!(
			"[WorkspaceService] Saving all documents (includeUntitled: {})",
			include_untitled
		);

		// Use DocumentProvider from MountainEnvironment
		let document_provider = self.environment.Require();

		match document_provider.SaveAll(include_untitled).await {
			Ok(_) => {
				info!("[WorkspaceService] All documents saved successfully");
				Ok(true)
			},
			Err(err) => {
				error!("[WorkspaceService] Failed to save all documents: {}", err);
				Err(Status::internal(format!("Failed to save all documents: {}", err)))
			},
		}
	}

	/// Apply text edits to a document
	///
	/// # Parameters
	/// - `uri`: The URI of the document to edit
	/// - `edits`: The text edits to apply
	///
	/// # Returns
	/// Success status indicating whether the edits were applied
	pub async fn apply_edit_impl(
		&self,
		uri: &Uri,
		edits: &[TextEdit],
	) -> Result<bool, Status> {
		let uri_value = &uri.value;
		debug!(
			"[WorkspaceService] Applying {} edits to document: {}",
			edits.len(),
			uri_value
		);

		// Use WorkspaceEditApplier from MountainEnvironment
		let edit_applier = self.environment.Require();

		// Convert TextEdit protobuf type to WorkspaceEditDTO format if needed
		// For now, we'll assume the provider can handle the list of edits
		match edit_applier.ApplyWorkspaceEdit(uri_value.to_string(), edits.to_vec()).await {
			Ok(_) => {
				info!("[WorkspaceService] Edits applied successfully to document: {}", uri_value);
				Ok(true)
			},
			Err(err) => {
				error!("[WorkspaceService] Failed to apply edits to document {}: {}", uri_value, err);
				Err(Status::internal(format!("Failed to apply edits: {}", err)))
			},
		}
	}

	// ==================== Configuration Operations ====================

	/// Update workspace configuration
	///
	/// This method is called when configuration values have changed,
	/// notifying Mountain and its components to update.
	///
	/// # Parameters
	/// - `changed_keys`: List of configuration keys that have changed
	///
	/// # Returns
	/// Success status
	pub async fn update_configuration_impl(
		&self,
		changed_keys: &[String],
	) -> Result<(), Status> {
		debug!(
			"[WorkspaceService] Updating configuration with {} changed keys",
			changed_keys.len()
		);

		// TODO: Implement configuration update
		// - Notify interested components of configuration changes
		// - Reload configuration values
		// - Trigger any necessary re-initialization

		Ok(())
	}

	// ==================== Workspace Folder Operations ====================

	/// Update workspace folders
	///
	/// # Parameters
	/// - `additions`: Folders to add
	/// - `removals`: Folders to remove
	///
	/// # Returns
	/// Success status
	pub async fn update_workspace_folders_impl(
		&self,
		additions: &[WorkspaceFolder],
		removals: &[WorkspaceFolder],
	) -> Result<(), Status> {
		info!(
			"[WorkspaceService] Updating workspace: {} additions, {} removals",
			additions.len(),
			removals.len()
		);

		// TODO: Implement workspace folder update
		// - Add new folders to workspace
		// - Remove specified folders
		// - Notify components of workspace changes
		// - Update FileSystem provider roots

		Ok(())
	}
}

#[cfg(test)]
mod tests {
	use super::*;

	// TODO: Add unit tests for WorkspaceService methods
	// These tests should mock the IPC and FileSystem layers
}
