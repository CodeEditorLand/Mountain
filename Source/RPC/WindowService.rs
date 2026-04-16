//! # WindowService - Advanced Window and UI Management
//!
//! This module provides a high-performance gRPC service for managing
//! window operations, document display, messages, and status bars.

use std::{
	collections::HashMap,
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

use crate::{
use crate::dev_log;
	Environment::MountainEnvironment::MountainEnvironment,
	RPC::WindowState::WindowState,
	Vine::Generated::{
		CreateWebviewPanelRequest,
		CreateWebviewPanelResponse,
		CreateWindowRequest,
		CreateWindowResponse,
		Empty,
		SetStatusBarTextRequest,
		SetWebviewHtmlRequest,
		ShowDocumentRequest,
		ShowDocumentResponse,
		ShowErrorRequest,
		ShowInformationRequest,
		ShowInputRequest,
		ShowInputResponse,
		ShowWarningRequest,
	},
};

#[cfg(feature = "Telemetry")]
pub struct WindowMetrics {
	window_create_counter:Counter<u64>,
	document_open_counter:Counter<u64>,
	message_show_counter:Counter<u64>,
	interaction_latency_histogram:Histogram<u64>,
}

#[cfg(feature = "Telemetry")]
impl WindowMetrics {
	pub fn new() -> Self {
		let meter = global::meter("WindowService");
		Self {
			window_create_counter:meter.u64_counter("windows_created").build(),
			document_open_counter:meter.u64_counter("documents_opened").build(),
			message_show_counter:meter.u64_counter("messages_shown").build(),
			interaction_latency_histogram:meter.u64_histogram("ui_interaction_latency_us").build(),
		}
	}

	pub fn record_window_created(&self, window_type:&str) {
		self.window_create_counter.add(1, &[KeyValue::new("type", window_type)]);
	}

	pub fn record_document_open(&self, language:Option<&str>) {
		self.document_open_counter
			.add(1, &[KeyValue::new("language", language.unwrap_or("unknown"))]);
	}

	pub fn record_message_shown(&self, severity:&str) {
		self.message_show_counter.add(1, &[KeyValue::new("severity", severity)]);
	}

	pub fn record_interaction(&self, operation:&str, latency_us:u64) {
		self.interaction_latency_histogram
			.record(latency_us, &[KeyValue::new("operation", operation)]);
	}
}

#[cfg(not(feature = "Telemetry"))]
pub struct WindowMetrics;

#[cfg(not(feature = "Telemetry"))]
impl WindowMetrics {
	pub fn new() -> Self { Self }
}

// ============ Window Service Implementation ============

pub struct WindowService {
	environment:MountainEnvironment,
	state_manager:Arc<WindowState>,
	metrics:WindowMetrics,
}

impl WindowService {
	pub fn Create(environment:MountainEnvironment, state_manager:Arc<WindowState>) -> Self {
		let metrics = WindowMetrics::new();
		dev_log!("grpc", "[WindowService] Initializing window service");
		Self { environment, state_manager, metrics }
	}

	pub async fn ShowInformation(&self, request:Request<ShowInformationRequest>) -> Result<Response<Empty>, Status> {
		let req = request.into_inner();
		#[cfg(feature = "Telemetry")]
		let _span = global::tracer("WindowService").start("ShowInformation");
		dev_log!("grpc", "[WindowService] Showing information message: {}", req.message);

		let window_provider = self.environment.Require();
		match window_provider.ShowInformation(req.message).await {
			Ok(_) => {
				#[cfg(feature = "Telemetry")]
				self.metrics.record_message_shown("info");
				Ok(Response::new(Empty {}))
			},
			Err(err) => {
				dev_log!("grpc", "error: [WindowService] Failed: {}", err);
				Err(Status::internal(format!("Failed: {}", err)))
			},
		}
	}

	pub async fn ShowWarning(&self, request:Request<ShowWarningRequest>) -> Result<Response<Empty>, Status> {
		let req = request.into_inner();
		dev_log!("grpc", "warn: [WindowService] Showing warning: {}", req.message);
		let window_provider = self.environment.Require();
		match window_provider.ShowWarning(req.message).await {
			Ok(_) => {
				#[cfg(feature = "Telemetry")]
				self.metrics.record_message_shown("warning");
				Ok(Response::new(Empty {}))
			},
			Err(err) => Err(Status::internal(format!("Failed: {}", err))),
		}
	}

	pub async fn ShowError(&self, request:Request<ShowErrorRequest>) -> Result<Response<Empty>, Status> {
		let req = request.into_inner();
		dev_log!("grpc", "error: [WindowService] Showing error: {}", req.message);
		let window_provider = self.environment.Require();
		match window_provider.ShowError(req.message).await {
			Ok(_) => {
				#[cfg(feature = "Telemetry")]
				self.metrics.record_message_shown("error");
				Ok(Response::new(Empty {}))
			},
			Err(err) => Err(Status::internal(format!("Failed: {}", err))),
		}
	}

	pub async fn ShowDocument(
		&self,
		request:Request<ShowDocumentRequest>,
	) -> Result<Response<ShowDocumentResponse>, Status> {
		let req = request.into_inner();
		#[cfg(feature = "Telemetry")]
		let span = global::tracer("WindowService").start("ShowDocument");
		dev_log!("grpc", "[WindowService] Opening document: {}", req.path);
		let start = Instant::now();
		let document_provider = self.environment.Require();
		match document_provider
			.ShowDocument(req.path, req.view_column, req.preserve_focus.unwrap_or(false))
			.await
		{
			Ok(handle) => {
				let elapsed = start.elapsed();
				#[cfg(feature = "Telemetry")]
				{
					span.set_attribute(KeyValue::new("duration_ms", elapsed.as_millis() as i64));
					span.end();
					self.metrics.record_document_open(req.language.as_deref());
					self.metrics.record_interaction("open_document", elapsed.as_micros() as u64);
				}
				Ok(Response::new(ShowDocumentResponse { handle }))
			},
			Err(err) => {
				#[cfg(feature = "Telemetry")]
				{
					span.end();
				}
				Err(Status::internal(format!("Failed: {}", err)))
			},
		}
	}

	pub async fn CreateWindow(
		&self,
		request:Request<CreateWindowRequest>,
	) -> Result<Response<CreateWindowResponse>, Status> {
		let req = request.into_inner();
		#[cfg(feature = "Telemetry")]
		let span = global::tracer("WindowService").start("CreateWindow");
		dev_log!("grpc", "[WindowService] Creating window: {:?}", req.window_type);
		let window_provider = self.environment.Require();
		match window_provider.CreateWindow(req.window_type, req.title).await {
			Ok(handle) => {
				#[cfg(feature = "Telemetry")]
				{
					span.end();
					self.metrics.record_window_created("new_window");
				}
				Ok(Response::new(CreateWindowResponse { handle }))
			},
			Err(err) => {
				#[cfg(feature = "Telemetry")]
				{
					span.end();
				}
				Err(Status::internal(format!("Failed: {}", err)))
			},
		}
	}

	pub async fn ShowInput(&self, request:Request<ShowInputRequest>) -> Result<Response<ShowInputResponse>, Status> {
		let req = request.into_inner();
		#[cfg(feature = "Telemetry")]
		let span = global::tracer("WindowService").start("ShowInput");
		dev_log!("grpc", "[WindowService] Showing input dialog: {}", req.prompt);
		let start = Instant::now();
		let window_provider = self.environment.Require();
		match window_provider
			.ShowInput(req.prompt, req.placeholder, req.default_value, req.password.unwrap_or(false))
			.await
		{
			Ok(result) => {
				let elapsed = start.elapsed();
				#[cfg(feature = "Telemetry")]
				{
					span.set_attribute(KeyValue::new("cancelled", result.value.is_none()));
					span.end();
					self.metrics.record_interaction("show_input", elapsed.as_micros() as u64);
				}
				Ok(Response::new(ShowInputResponse {
					value:result.value.unwrap_or_default(),
					cancelled:result.cancelled,
				}))
			},
			Err(err) => {
				#[cfg(feature = "Telemetry")]
				{
					span.end();
				}
				Err(Status::internal(format!("Failed: {}", err)))
			},
		}
	}

	pub async fn SetStatusBarText(&self, request:Request<SetStatusBarTextRequest>) -> Result<Response<Empty>, Status> {
		let req = request.into_inner();
		let window_provider = self.environment.Require();
		match window_provider
			.SetStatusBarText(req.text, req.position, req.priority.unwrap_or(0))
			.await
		{
			Ok(_) => Ok(Response::new(Empty {})),
			Err(err) => Err(Status::internal(format!("Failed: {}", err))),
		}
	}

	pub async fn CreateWebviewPanel(
		&self,
		request:Request<CreateWebviewPanelRequest>,
	) -> Result<Response<CreateWebviewPanelResponse>, Status> {
		let req = request.into_inner();
		let webview_provider = self.environment.Require();
		let handle = self.state_manager.next_webview_handle();
		let view_column = req.view_column.unwrap_or(1) as i32;
		match webview_provider
			.CreateWebviewPanel(
				handle,
				req.view_type,
				req.title,
				if req.icon_path.is_empty() { None } else { Some(req.icon_path) },
				view_column,
				req.preserve_focus.unwrap_or(false),
				req.enable_find_widget.unwrap_or(false),
				req.retain_context_when_hidden.unwrap_or(true),
				req.local_resource_roots,
			)
			.await
		{
			Ok(_) => Ok(Response::new(CreateWebviewPanelResponse { handle })),
			Err(err) => Err(Status::internal(format!("Failed: {}", err))),
		}
	}

	pub async fn SetWebviewHtml(&self, request:Request<SetWebviewHtmlRequest>) -> Result<Response<Empty>, Status> {
		let req = request.into_inner();
		let webview_provider = self.environment.Require();
		match webview_provider.SetWebviewHtml(req.handle, req.html).await {
			Ok(_) => Ok(Response::new(Empty {})),
			Err(err) => Err(Status::internal(format!("Failed: {}", err))),
		}
	}

	pub async fn OnDidReceiveMessage(&self, handle:u32, message:&str) -> Result<(), Status> {
		dev_log!("grpc", "[WindowService] Received webview message from {}: {}", handle, message);
		// DEPENDENCY: Forward to extension handler - requires ExtensionHandler
		// implementation in Wind/Sky frontend communication layer
		Ok(())
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	// DEPENDENCY: Tests require full WindowService implementation including:
	// - Window creation/destruction lifecycle
	// - Message forwarding to extension handlers
	// - State management integration
}
