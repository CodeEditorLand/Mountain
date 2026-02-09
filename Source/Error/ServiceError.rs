//! # Service Error Types
//!
//! Provides service-related error types for Mountain.
//! Used for all service operation errors.

use std::{error::Error as StdError, fmt};

use serde::{Deserialize, Serialize};

use super::CoreError::{ErrorContext, ErrorKind, ErrorSeverity, MountainError, Result};

/// Service operation error types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ServiceError {
	/// Service not found
	ServiceNotFound { context:ErrorContext, service_name:String },
	/// Service initialization failed
	InitializationFailed { context:ErrorContext, service_name:String, source:Option<String> },
	/// Service already running
	AlreadyRunning { context:ErrorContext, service_name:String },
	/// Service not running
	NotRunning { context:ErrorContext, service_name:String },
	/// Service start failed
	StartFailed { context:ErrorContext, service_name:String, source:Option<String> },
	/// Service stop failed
	StopFailed { context:ErrorContext, service_name:String, source:Option<String> },
	/// Service timeout
	Timeout { context:ErrorContext, service_name:String, operation:String, timeout_ms:u64 },
	/// Service dependency error
	DependencyError { context:ErrorContext, service_name:String, dependency:String },
}

impl ServiceError {
	/// Get the error context
	pub fn context(&self) -> &ErrorContext {
		match self {
			ServiceError::ServiceNotFound { context, .. } => context,
			ServiceError::InitializationFailed { context, .. } => context,
			ServiceError::AlreadyRunning { context, .. } => context,
			ServiceError::NotRunning { context, .. } => context,
			ServiceError::StartFailed { context, .. } => context,
			ServiceError::StopFailed { context, .. } => context,
			ServiceError::Timeout { context, .. } => context,
			ServiceError::DependencyError { context, .. } => context,
		}
	}

	/// Create a service not found error
	pub fn service_not_found(service_name:impl Into<String>) -> Self {
		let service_name_str = service_name.into();
		Self::ServiceNotFound {
			context:ErrorContext::new(format!("Service not found: {}", service_name_str))
				.with_kind(ErrorKind::Service)
				.with_severity(ErrorSeverity::Error),
			service_name:service_name_str,
		}
	}

	/// Create an initialization failed error
	pub fn initialization_failed(service_name:impl Into<String>, source:Option<String>) -> Self {
		let service_name_str = service_name.into();
		Self::InitializationFailed {
			context:ErrorContext::new(format!("Service initialization failed: {}", service_name_str))
				.with_kind(ErrorKind::Service)
				.with_severity(ErrorSeverity::Critical),
			service_name:service_name_str,
			source,
		}
	}

	/// Create an already running error
	pub fn already_running(service_name:impl Into<String>) -> Self {
		let service_name_str = service_name.into();
		Self::AlreadyRunning {
			context:ErrorContext::new(format!("Service already running: {}", service_name_str))
				.with_kind(ErrorKind::Service)
				.with_severity(ErrorSeverity::Warning),
			service_name:service_name_str,
		}
	}

	/// Create a not running error
	pub fn not_running(service_name:impl Into<String>) -> Self {
		let service_name_str = service_name.into();
		Self::NotRunning {
			context:ErrorContext::new(format!("Service not running: {}", service_name_str))
				.with_kind(ErrorKind::Service)
				.with_severity(ErrorSeverity::Error),
			service_name:service_name_str,
		}
	}

	/// Create a start failed error
	pub fn start_failed(service_name:impl Into<String>, source:Option<String>) -> Self {
		let service_name_str = service_name.into();
		Self::StartFailed {
			context:ErrorContext::new(format!("Service start failed: {}", service_name_str))
				.with_kind(ErrorKind::Service)
				.with_severity(ErrorSeverity::Error),
			service_name:service_name_str,
			source,
		}
	}

	/// Create a timeout error
	pub fn timeout(service_name:impl Into<String>, operation:impl Into<String>, timeout_ms:u64) -> Self {
		let service_name_str = service_name.into();
		let operation_str = operation.into();
		Self::Timeout {
			context:ErrorContext::new(format!(
				"Service timeout: {} operation timed out after {}ms",
				service_name_str, timeout_ms
			))
			.with_kind(ErrorKind::Service)
			.with_severity(ErrorSeverity::Error)
			.with_operation(operation_str.clone()),
			service_name:service_name_str,
			operation:operation_str,
			timeout_ms,
		}
	}

	/// Create a dependency error
	pub fn dependency_error(service_name:impl Into<String>, dependency:impl Into<String>) -> Self {
		let service_name_str = service_name.into();
		let dependency_str = dependency.into();
		Self::DependencyError {
			context:ErrorContext::new(format!(
				"Service dependency error: {} depends on {}",
				service_name_str, dependency_str
			))
			.with_kind(ErrorKind::Service)
			.with_severity(ErrorSeverity::Critical),
			service_name:service_name_str,
			dependency:dependency_str,
		}
	}
}

impl fmt::Display for ServiceError {
	fn fmt(&self, f:&mut fmt::Formatter<'_>) -> fmt::Result { write!(f, "{}", self.context()) }
}

impl StdError for ServiceError {}

impl From<ServiceError> for MountainError {
	fn from(err:ServiceError) -> Self { MountainError::new(err.context().clone()).with_source(err.to_string()) }
}
