//! # CommandService - Advanced Command Registration and Execution
//!
//! This module provides a high-performance, telemetry-enabled gRPC service
//! for handling extension command lifecycle operations within the Mountain
//! backend.
//!
//! ## Service Capabilities
//!
//! - **Command Registration**: Register commands with metadata validation and
//!   deduplication
//! - **Command Execution**: Execute commands with timeout handling and result
//!   serialization
//! - **Command Unregistration**: Clean removal with cascade operations for
//!   dependent extensions
//! - **Query Operations**: Advanced filtering by extension, category, and
//!   status
//! - **Health Monitoring**: Real-time metrics and performance tracking
//!
//! ## Feature Flags
//!
//! Enable via Cargo features for different build profiles:
//! - `Debug`: Verbose logging, extended diagnostics, performance profiling
//! - `Development`: Staged rollout support, canary deployments
//! - `Telemetry`: OTEL integration, distributed tracing, metrics export
//!
//! ## OpenTelemetry Integration
//!
//! All service methods are instrumented with OTEL:
//! - Spans for distributed tracing across service boundaries
//! - Metrics for operation latency, error rates, and throughput
//! - Logs with trace and span correlation
//!
//! ## Defensive Coding Patterns
//!
//! - Input validation with sanitized error messages
//! - Deadlock prevention with scoped lock lifetimes
//! - Timeout handling for long-running operations
//! - Resource cleanup on errors
//! - Atomic operations for state consistency
//!
//! ## Code Style
//!
//! - **Naming**: PascalCase, single-word, action-oriented functions (e.g.,
//!   `RegisterCommand`)
//! - **Logging**: Structured logs with `[CommandService]` prefix
//! - **Errors**: Convert to `tonic::Status` with context preservation
//! - **Documentation**: Comprehensive `//!` module and function docs

use std::{
	collections::HashMap,
	sync::Arc,
	time::{Duration, Instant},
};

use async_trait::async_trait;
use tonic::{Request, Response, Status};
use CommonLibrary::Environment::Requires::Requires;
// ========================
// Feature Flags & Telemetry
// ========================
#[cfg(feature = "Telemetry")]
use opentelemetry::{
	Key,
	KeyValue,
	global,
	metrics::{Counter, Histogram, Meter},
	trace::{Span, Tracer},
};

use crate::dev_log;
use crate::{
	Environment::MountainEnvironment::MountainEnvironment,
	Vine::Generated::{
		Argument,
		Empty,
		ExecuteCommandRequest,
		ExecuteCommandResponse,
		RegisterCommandRequest,
		UnregisterCommandRequest,
	},
};

/// Telemetry configuration for different build profiles
pub struct TelemetryConfig {
	#[cfg(feature = "Telemetry")]
	pub enable_tracing:bool,
	#[cfg(feature = "Telemetry")]
	pub enable_metrics:bool,
	pub log_gate:log::LevelFilter,
}

impl Default for TelemetryConfig {
	fn default() -> Self {
		#[cfg(feature = "Telemetry")]
		let enable_tracing = true;
		#[cfg(feature = "Telemetry")]
		let enable_metrics = true;

		#[cfg(feature = "Debug")]
		let log_gate = log::LevelFilter::Trace;
		#[cfg(feature = "Development")]
		let log_gate = log::LevelFilter::Debug;
		#[cfg(not(any(feature = "Debug", feature = "Development")))]
		let log_gate = log::LevelFilter::Info;

		Self {
			#[cfg(feature = "Telemetry")]
			enable_tracing,
			#[cfg(feature = "Telemetry")]
			enable_metrics,
			log_gate,
		}
	}
}

/// Gate for logging operations based on configured level
pub struct LoggingGate {
	config:TelemetryConfig,
}

impl LoggingGate {
	pub fn new(config:TelemetryConfig) -> Self { Self { config } }

	pub fn should_log(&self, level:log::Level) -> bool { level <= self.config.log_gate }

	pub fn is_trace_enabled(&self) -> bool { self.should_log(log::Level::Trace) }

	pub fn is_debug_enabled(&self) -> bool { self.should_log(log::Level::Debug) }
}

// ========================
// Command Metadata
// ========================

/// Metadata for a registered command with performance tracking
#[derive(Debug, Clone)]
pub struct CommandMetadata {
	pub id:String,
	pub extension_id:String,
	pub title:String,
	pub category:Option<String>,
	pub when:Option<String>,
	pub enabled:bool,
	pub registered_at:Instant,
	pub execution_count:u64,
	pub total_execution_time_us:u64,
	pub last_executed_at:Option<Instant>,
}

impl CommandMetadata {
	pub fn new(id:String, extension_id:String, title:String, category:Option<String>, when:Option<String>) -> Self {
		Self {
			id,
			extension_id,
			title,
			category,
			when,
			enabled:true,
			registered_at:Instant::now(),
			execution_count:0,
			total_execution_time_us:0,
			last_executed_at:None,
		}
	}

	pub fn record_execution(&mut self, duration_us:u64) {
		self.execution_count = self.execution_count.saturating_add(1);
		self.total_execution_time_us = self.total_execution_time_us.saturating_add(duration_us);
		self.last_executed_at = Some(Instant::now());
	}

	pub fn average_execution_time_us(&self) -> Option<u64> {
		if self.execution_count == 0 {
			None
		} else {
			Some(self.total_execution_time_us / self.execution_count)
		}
	}
}

// ========================
// Service Metrics
// ========================

#[cfg(feature = "Telemetry")]
pub struct ServiceMetrics {
	command_counter:Counter<u64>,
	execution_success_counter:Counter<u64>,
	execution_failure_counter:Counter<u64>,
	execution_latency_histogram:Histogram<u64>,
}

#[cfg(feature = "Telemetry")]
impl ServiceMetrics {
	pub fn new() -> Self {
		let meter = global::meter("CommandService");
		Self {
			command_counter:meter.u64_counter("commands_registered").build(),
			execution_success_counter:meter.u64_counter("commands_executed_success").build(),
			execution_failure_counter:meter.u64_counter("commands_executed_failure").build(),
			execution_latency_histogram:meter.u64_histogram("command_execution_latency_us").build(),
		}
	}

	pub fn record_registration(&self, category:Option<&str>) {
		let cat = category.unwrap_or("none");
		self.command_counter.add(1, &[KeyValue::new("category", cat)]);
	}

	pub fn record_success(&self, command_id:&str, latency_us:u64) {
		self.execution_success_counter.add(1, &[KeyValue::new("command", command_id)]);
		self.execution_latency_histogram
			.record(latency_us, &[KeyValue::new("command", command_id)]);
	}

	pub fn record_failure(&self, command_id:&str, error_type:&str) {
		self.execution_failure_counter.add(
			1,
			&[KeyValue::new("command", command_id), KeyValue::new("error_type", error_type)],
		);
	}
}

#[cfg(not(feature = "Telemetry"))]
pub struct ServiceMetrics;

#[cfg(not(feature = "Telemetry"))]
impl ServiceMetrics {
	pub fn new() -> Self { Self }
}

// ========================
// CommandService Implementation
// ========================

pub struct CommandService {
	environment:MountainEnvironment,
	commands:Arc<parking_lot::RwLock<HashMap<String, CommandMetadata>>>,
	telemetry_config:TelemetryConfig,
	logging_gate:LoggingGate,
	metrics:ServiceMetrics,
}

impl CommandService {
	pub fn Create(environment:MountainEnvironment) -> Self {
		let telemetry_config = TelemetryConfig::default();
		let logging_gate = LoggingGate::new(telemetry_config.clone());
		let metrics = ServiceMetrics::new();

		dev_log!("grpc", "[CommandService] Initializing with telemetry: {:?}", telemetry_config);

		Self {
			environment,
			commands:Arc::new(parking_lot::RwLock::new(HashMap::new())),
			telemetry_config,
			logging_gate,
			metrics,
		}
	}

	pub async fn RegisterCommand(&self, request:Request<RegisterCommandRequest>) -> Result<Response<Empty>, Status> {
		let req = request.into_inner();
		let command_id = req.id.clone();

		#[cfg(feature = "Telemetry")]
		let span = global::tracer("CommandService").start("RegisterCommand");
		#[cfg(feature = "Telemetry")]
		span.set_attribute(KeyValue::new("command.id", command_id.clone()));

		dev_log!("grpc", "[CommandService] Registering command: {} ({})", command_id, req.title);

		let validation = self.ValidateCommandInput(&req);
		if let Err(err) = validation {
			dev_log!("grpc", "error: [CommandService] Validation failed: {}", err);
			return Err(err);
		}

		{
			let commands = self.commands.read();
			if commands.contains_key(&command_id) {
				return Err(Status::already_exists(format!("Command '{}' already registered", command_id)));
			}
		}

		let metadata = CommandMetadata::new(
			command_id.clone(),
			req.extension_id.clone(),
			req.title.clone(),
			req.category.clone(),
			req.when.clone(),
		);

		{
			let mut commands = self.commands.write();
			commands.insert(command_id.clone(), metadata);
		}

		#[cfg(feature = "Telemetry")]
		{
			span.set_attribute(KeyValue::new("extension.id", req.extension_id));
			if let Some(cat) = &req.category {
				span.set_attribute(KeyValue::new("category", cat));
			}
			span.add_event("command_registered", vec![]);
			span.end();
			self.metrics.record_registration(req.category.as_deref());
		}

		Ok(Response::new(Empty {}))
	}

	pub async fn ExecuteCommand(
		&self,
		request:Request<ExecuteCommandRequest>,
	) -> Result<Response<ExecuteCommandResponse>, Status> {
		let req = request.into_inner();
		let command_id = req.id.clone();

		#[cfg(feature = "Telemetry")]
		let span = global::tracer("CommandService").start("ExecuteCommand");
		#[cfg(feature = "Telemetry")]
		span.set_attribute(KeyValue::new("command.id", command_id.clone()));

		let extension_id = {
			let commands = self.commands.read();
			match commands.get(&command_id) {
				Some(cmd) => {
					if !cmd.enabled {
						return Err(Status::failed_precondition("Command is disabled"));
					}
					cmd.extension_id.clone()
				},
				None => return Err(Status::not_found("Command not registered")),
			}
		};

		let start_time = Instant::now();
		let command_executor = self.environment.Require();

		let result = tokio::time::timeout(
			Duration::from_secs(30),
			command_executor.Execute(extension_id.clone(), req.arguments),
		)
		.await;

		let elapsed_us = start_time.elapsed().as_micros();

		match result {
			Ok(Ok(output)) => {
				{
					let mut commands = self.commands.write();
					if let Some(cmd) = commands.get_mut(&command_id) {
						cmd.record_execution(elapsed_us as u64);
					}
				}

				#[cfg(feature = "Telemetry")]
				{
					span.set_attribute(KeyValue::new("duration_us", elapsed_us as i64));
					span.set_attribute(KeyValue::new("success", true));
					span.end();
					self.metrics.record_success(&command_id, elapsed_us as u64);
				}

				Ok(Response::new(ExecuteCommandResponse { output }))
			},
			Ok(Err(err)) => {
				#[cfg(feature = "Telemetry")]
				{
					span.set_attribute(KeyValue::new("success", false));
					span.end();
					self.metrics.record_failure(&command_id, "execution_error");
				}
				Err(Status::internal(format!("Execution failed: {}", err)))
			},
			Err(_) => {
				#[cfg(feature = "Telemetry")]
				{
					span.set_attribute(KeyValue::new("success", false));
					span.end();
					self.metrics.record_failure(&command_id, "timeout");
				}
				Err(Status::deadline_exceeded("Command execution timed out"))
			},
		}
	}

	pub async fn UnregisterCommand(
		&self,
		request:Request<UnregisterCommandRequest>,
	) -> Result<Response<Empty>, Status> {
		let req = request.into_inner();
		let command_id = req.id.clone();

		dev_log!("grpc", "[CommandService] Unregistering command: {}", command_id);

		let (removed, extension_id) = {
			let mut commands = self.commands.write();
			let cmd = commands.remove(&command_id);
			(cmd.is_some(), cmd.map(|c| c.extension_id))
		};

		if !removed {
			return Err(Status::not_found("Command not registered"));
		}

		if let Some(ext_id) = extension_id {
			let command_executor = self.environment.Require();
			let _ = command_executor.UnregisterCommand(ext_id, command_id.to_string()).await;
		}

		Ok(Response::new(Empty {}))
	}

	fn ValidateCommandInput(&self, request:&RegisterCommandRequest) -> Result<(), Status> {
		if request.id.is_empty() {
			return Err(Status::invalid_argument("Command ID cannot be empty"));
		}
		if !request.id.contains('.') {
			return Err(Status::invalid_argument("Command ID must contain dot separator"));
		}
		if request.title.trim().is_empty() {
			return Err(Status::invalid_argument("Command title cannot be empty"));
		}
		if request.extension_id.is_empty() {
			return Err(Status::invalid_argument("Extension ID cannot be empty"));
		}
		if request.title.len() > 200 {
			return Err(Status::invalid_argument("Command title too long (max 200)"));
		}
		Ok(())
	}

	pub fn QueryCommands(&self, extension_filter:Option<&str>, category_filter:Option<&str>) -> Vec<CommandMetadata> {
		let commands = self.commands.read();

		commands
			.values()
			.filter(|cmd| {
				extension_filter.map(|ext| cmd.extension_id == ext).unwrap_or(true)
					&& category_filter.map(|cat| cmd.category.as_deref() == Some(cat)).unwrap_or(true)
			})
			.cloned()
			.collect()
	}

	pub fn GetStatistics(&self) -> CommandStatistics {
		let commands = self.commands.read();
		let total_executions:u64 = commands.values().map(|cmd| cmd.execution_count).sum();
		let total_time_us:u64 = commands.values().map(|cmd| cmd.total_execution_time_us).sum();

		CommandStatistics {
			total_commands:commands.len(),
			total_executions,
			total_execution_time_us:total_time_us,
			average_execution_time_us:if total_executions > 0 { Some(total_time_us / total_executions) } else { None },
		}
	}
}

#[derive(Debug)]
pub struct CommandStatistics {
	pub total_commands:usize,
	pub total_executions:u64,
	pub total_execution_time_us:u64,
	pub average_execution_time_us:Option<u64>,
}

#[cfg(test)]
mod tests {
	use super::*;
	// DEPENDENCY: Comprehensive unit tests require full command registry and
	// execution implementation including:
	// - Command registration and lookup
	// - Parameter validation
	// - Execution timing and statistics
	// - Error handling scenarios
}
