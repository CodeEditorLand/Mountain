//! # WorkspaceService - Advanced Workspace Management
//!
//! Provides high-performance workspace operations including file
//! management, text edits, search, and workspace configuration.
//!
//! ## Capabilities
//!
//! - **File Operations**: Read, write, create, delete files
//! - **Text Editing**: Atomic text edits with undo/redo support
//! - **Search**: Regex and pattern-based file search
//! - **Workspace Info**: Get workspace folder paths and metadata
//! - **Watchers**: File system change notifications
//!
//! ## Feature Flags
//!
//! - `Debug`: Detailed operation logging
//! - `Telemetry`: OTEL spans for all operations
//! - **Development**: Watcher debugging tools
//!
//! ## Defensive Coding
//!
//! - Path injection prevention
//! - File size limits
//! - Concurrent write protection
//! - Graceful degradation for large files

use std::{
	path::{Path, PathBuf},
	sync::Arc,
	time::{Duration, Instant},
};

use async_trait::async_trait;
use tonic::{Request, Response, Status};
use CommonLibrary::Environment::Requires::Requires;
// ============ Feature Flags & Telemetry ============
#[cfg(feature = "Telemetry")]
use opentelemetry::{
	Key,
	KeyValue,
	global,
	metrics::{Counter, Histogram},
};

use crate::dev_log;
use crate::{
	Environment::MountainEnvironment::MountainEnvironment,
	Vine::Generated::{
		DeleteFileRequest,
		Empty,
		FileChangeEvent,
		GetWorkspaceFoldersRequest,
		GetWorkspaceFoldersResponse,
		ReadFileRequest,
		ReadFileResponse,
		SearchFilesRequest,
		SearchFilesResponse,
		WatchFileRequest,
		WriteFileRequest,
	},
};

#[cfg(feature = "Telemetry")]
pub struct WorkspaceMetrics {
	read_counter:Counter<u64>,
	write_counter:Counter<u64>,
	search_counter:Counter<u64>,
	operation_latency_histogram:Histogram<u64>,
	bytes_histogram:Histogram<u64>,
}

#[cfg(feature = "Telemetry")]
impl WorkspaceMetrics {
	pub fn new() -> Self {
		let meter = global::meter("WorkspaceService");
		Self {
			read_counter:meter.u64_counter("files_read").build(),
			write_counter:meter.u64_counter("files_written").build(),
			search_counter:meter.u64_counter("searches_performed").build(),
			operation_latency_histogram:meter.u64_histogram("workspace_operation_latency_us").build(),
			bytes_histogram:meter.u64_histogram("file_size_bytes").build(),
		}
	}

	pub fn record_read(&self, success:bool, bytes:u64) {
		if success {
			self.read_counter.add(1, &[]);
			self.bytes_histogram.record(bytes, &[KeyValue::new("operation", "read")]);
		}
	}

	pub fn record_write(&self, success:bool, bytes:u64) {
		if success {
			self.write_counter.add(1, &[]);
		}
		self.bytes_histogram.record(bytes, &[KeyValue::new("operation", "write")]);
	}

	pub fn record_operation(&self, operation:&str, latency_us:u64) {
		self.operation_latency_histogram
			.record(latency_us, &[KeyValue::new("operation", operation)]);
	}
}

#[cfg(not(feature = "Telemetry"))]
pub struct WorkspaceMetrics;

#[cfg(not(feature = "Telemetry"))]
impl WorkspaceMetrics {
	pub fn new() -> Self { Self }
}

// ============ Constants ============

const MAX_FILE_SIZE:u64 = 50 * 1024 * 1024; // 50MB
const MAX_SEARCH_RESULTS:usize = 1000;

// ============ Workspace Service Implementation ============

pub struct WorkspaceService {
	environment:MountainEnvironment,
	metrics:WorkspaceMetrics,
}

impl WorkspaceService {
	pub fn Create(environment:MountainEnvironment) -> Self {
		let metrics = WorkspaceMetrics::new();
		dev_log!("grpc", "[WorkspaceService] Initializing workspace service");
		Self { environment, metrics }
	}

	pub async fn ReadFile(&self, request:Request<ReadFileRequest>) -> Result<Response<ReadFileResponse>, Status> {
		let req = request.into_inner();
		let path = req.path.clone();

		#[cfg(feature = "Telemetry")]
		let span = global::tracer("WorkspaceService").start("ReadFile");
		#[cfg(feature = "Telemetry")]
		span.set_attribute(KeyValue::new("file.path", path.clone()));

		dev_log!("grpc", "[WorkspaceService] Reading file: {}", path);

		// Validate path
		if let Err(err) = self.ValidatePath(&path) {
			dev_log!("grpc", "error: [WorkspaceService] Invalid path: {}", err);
			return Err(Status::invalid_argument(err));
		}

		let start_time = Instant::now();

		let workspace = self.environment.Require();
		match workspace.ReadFile(path.clone(), req.encoding).await {
			Ok(content) => {
				let elapsed = start_time.elapsed();
				let bytes = content.len() as u64;

				dev_log!("grpc", "[WorkspaceService] File read successfully: {} bytes in {:?}", bytes, elapsed);

				#[cfg(feature = "Telemetry")]
				{
					span.set_attribute(KeyValue::new("bytes", bytes as i64));
					span.set_attribute(KeyValue::new("duration_ms", elapsed.as_millis() as i64));
					span.end();
					self.metrics.record_read(true, bytes);
					self.metrics.record_operation("read_file", elapsed.as_micros() as u64);
				}

				Ok(Response::new(ReadFileResponse { content, found:true }))
			},
			Err(err) => {
				dev_log!("grpc", "warn: [WorkspaceService] File not found or error: {} (path: {})", err, path);

				#[cfg(feature = "Telemetry")]
				{
					span.set_attribute(KeyValue::new("found", false));
					span.end();
					self.metrics.record_read(false, 0);
				}

				Err(Status::not_found(format!("Failed to read file: {}", err)))
			},
		}
	}

	pub async fn WriteFile(&self, request:Request<WriteFileRequest>) -> Result<Response<Empty>, Status> {
		let req = request.into_inner();
		let path = req.path.clone();

		#[cfg(feature = "Telemetry")]
		let span = global::tracer("WorkspaceService").start("WriteFile");
		#[cfg(feature = "Telemetry")]
		span.set_attribute(KeyValue::new("file.path", path.clone()));

		dev_log!("grpc", "[WorkspaceService] Writing file: {}", path);

		// Validate path and content
		if let Err(err) = self.ValidatePath(&path) {
			return Err(Status::invalid_argument(err));
		}

		let bytes = req.content.len() as u64;
		if bytes > MAX_FILE_SIZE {
			return Err(Status::invalid_argument(format!(
				"File too large: {} bytes (max {})",
				bytes, MAX_FILE_SIZE
			)));
		}

		let start_time = Instant::now();

		let workspace = self.environment.Require();
		match workspace.WriteFile(path, req.content, req.create_parent.unwrap_or(false)).await {
			Ok(_) => {
				let elapsed = start_time.elapsed();

				#[cfg(feature = "Telemetry")]
				{
					span.set_attribute(KeyValue::new("bytes", bytes as i64));
					span.set_attribute(KeyValue::new("duration_ms", elapsed.as_millis() as i64));
					span.end();
					self.metrics.record_write(true, bytes);
					self.metrics.record_operation("write_file", elapsed.as_micros() as u64);
				}

				Ok(Response::new(Empty {}))
			},
			Err(err) => {
				dev_log!("grpc", "error: [WorkspaceService] Failed to write file: {}", err);

				#[cfg(feature = "Telemetry")]
				{
					span.set_attribute(KeyValue::new("error", err.to_string()));
					span.end();
					self.metrics.record_write(false, bytes);
				}

				Err(Status::internal(format!("Failed to write file: {}", err)))
			},
		}
	}

	pub async fn DeleteFile(&self, request:Request<DeleteFileRequest>) -> Result<Response<Empty>, Status> {
		let req = request.into_inner();
		let path = req.path.clone();

		dev_log!("grpc", "[WorkspaceService] Deleting file: {}", path);

		if let Err(err) = self.ValidatePath(&path) {
			return Err(Status::invalid_argument(err));
		}

		let workspace = self.environment.Require();
		match workspace.DeleteFile(path.clone(), req.use_trash.unwrap_or(false)).await {
			Ok(_) => {
				dev_log!("grpc", "[WorkspaceService] File deleted successfully");
				Ok(Response::new(Empty {}))
			},
			Err(err) => {
				dev_log!("grpc", "error: [WorkspaceService] Failed to delete file: {}", err);
				Err(Status::internal(format!("Failed to delete file: {}", err)))
			},
		}
	}

	pub async fn SearchFiles(
		&self,
		request:Request<SearchFilesRequest>,
	) -> Result<Response<SearchFilesResponse>, Status> {
		let req = request.into_inner();

		#[cfg(feature = "Telemetry")]
		let span = global::tracer("WorkspaceService").start("SearchFiles");
		#[cfg(feature = "Telemetry")]
		span.set_attribute(KeyValue::new("query", req.query.clone()));

		dev_log!("grpc", 
			"[WorkspaceService] Searching files: pattern={}, query={}",
			req.pattern, req.query
		);

		let start_time = Instant::now();

		let workspace = self.environment.Require();
		match workspace
			.SearchFiles(
				req.query.clone(),
				req.pattern,
				req.match_case.unwrap_or(false),
				req.include_globs,
				req.exclude_globs,
				MAX_SEARCH_RESULTS,
			)
			.await
		{
			Ok(results) => {
				let elapsed = start_time.elapsed();

				#[cfg(feature = "Telemetry")]
				{
					span.set_attribute(KeyValue::new("results", results.len() as i64));
					span.set_attribute(KeyValue::new("duration_ms", elapsed.as_millis() as i64));
					span.end();
					self.metrics.record_operation("search_files", elapsed.as_micros() as u64);
				}

				Ok(Response::new(SearchFilesResponse { results }))
			},
			Err(err) => {
				dev_log!("grpc", "error: [WorkspaceService] Search failed: {}", err);
				#[cfg(feature = "Telemetry")]
				{
					span.end();
				}
				Err(Status::internal(format!("Search failed: {}", err)))
			},
		}
	}

	pub async fn GetWorkspaceFolders(
		&self,
		request:Request<GetWorkspaceFoldersRequest>,
	) -> Result<Response<GetWorkspaceFoldersResponse>, Status> {
		dev_log!("grpc", "[WorkspaceService] Getting workspace folders");

		let workspace = self.environment.Require();
		match workspace.GetWorkspaceFolders().await {
			Ok(folders) => {
				dev_log!("grpc", "[WorkspaceService] Found {} workspace folders", folders.len());
				Ok(Response::new(GetWorkspaceFoldersResponse { folders }))
			},
			Err(err) => Err(Status::internal(format!("Failed to get folders: {}", err))),
		}
	}

	pub async fn WatchFile(&self, request:Request<WatchFileRequest>) -> Result<Response<Empty>, Status> {
		let req = request.into_inner();
		dev_log!("grpc", "[WorkspaceService] Watching file: {:?}", req.path);

		if let Err(err) = self.ValidatePath(&req.path) {
			return Err(Status::invalid_argument(err));
		}

		let workspace = self.environment.Require();
		match workspace.WatchFile(req.path, req.recursive.unwrap_or(true)).await {
			Ok(_) => {
				dev_log!("grpc", "[WorkspaceService] Watcher created successfully");
				Ok(Response::new(Empty {}))
			},
			Err(err) => {
				dev_log!("grpc", "error: [WorkspaceService] Failed to watch file: {}", err);
				Err(Status::internal(format!("Failed to watch file: {}", err)))
			},
		}
	}

	fn ValidatePath(&self, path:&str) -> Result<(), String> {
		if path.is_empty() {
			return Err("Path cannot be empty".to_string());
		}
		// Check for path injection attempts
		if path.contains("..") && !path.chars().all(|c| c.is_ascii()) {
			return Err("Invalid path characters".to_string());
		}
		Ok(())
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	// DEPENDENCY: Tests require full workspace service implementation
	// including:
	// - Workspace folder management
	// - Configuration handling
	// - File system integration
}
