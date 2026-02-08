//! # Provider Error Types
//!
//! Provides provider-specific error types for Mountain.
//! Used for all provider related errors (DocumentProvider, FileSystemProvider, etc.).

use super::CoreError::{ErrorContext, ErrorKind, ErrorSeverity, MountainError, Result};
use serde::{Deserialize, Serialize};
use std::error::Error as StdError;
use std::fmt;

/// Provider operation error types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ProviderError {
    /// Provider not registered
    ProviderNotRegistered {
        context: ErrorContext,
        provider_name: String,
    },
    /// Provider initialization failed
    InitializationFailed {
        context: ErrorContext,
        provider_name: String,
        source: Option<String>,
    },
    /// Provider method not implemented
    MethodNotImplemented {
        context: ErrorContext,
        provider_name: String,
        method_name: String,
    },
    /// Invalid provider configuration
    InvalidConfiguration {
        context: ErrorContext,
        provider_name: String,
        errors: Vec<String>,
    },
    /// Provider timeout
    Timeout {
        context: ErrorContext,
        provider_name: String,
        operation: String,
        timeout_ms: u64,
    },
    /// Provider unavailable
    Unavailable {
        context: ErrorContext,
        provider_name: String,
        reason: String,
    },
}

impl ProviderError {
    /// Get the error context
    pub fn context(&self) -> &ErrorContext {
        match self {
            ProviderError::ProviderNotRegistered { context, .. } => context,
            ProviderError::InitializationFailed { context, .. } => context,
            ProviderError::MethodNotImplemented { context, .. } => context,
            ProviderError::InvalidConfiguration { context, .. } => context,
            ProviderError::Timeout { context, .. } => context,
            ProviderError::Unavailable { context, .. } => context,
        }
    }

    /// Create a provider not registered error
    pub fn provider_not_registered(provider_name: impl Into<String>) -> Self {
        Self::ProviderNotRegistered {
            context: ErrorContext::new(format!("Provider not registered: {}", provider_name.into()))
                .with_kind(ErrorKind::Provider)
                .with_severity(ErrorSeverity::Error),
            provider_name: provider_name.into(),
        }
    }

    /// Create an initialization failed error
    pub fn initialization_failed(provider_name: impl Into<String>, source: Option<String>) -> Self {
        Self::InitializationFailed {
            context: ErrorContext::new(format!("Provider initialization failed: {}", provider_name.into()))
                .with_kind(ErrorKind::Provider)
                .with_severity(ErrorSeverity::Critical),
            provider_name: provider_name.into(),
            source,
        }
    }

    /// Create a method not implemented error
    pub fn method_not_implemented(provider_name: impl Into<String>, method_name: impl Into<String>) -> Self {
        Self::MethodNotImplemented {
            context: ErrorContext::new(format!("Method '{}' not implemented in provider '{}'", method_name.into(), provider_name.into()))
                .with_kind(ErrorKind::Provider)
                .with_severity(ErrorSeverity::Error),
            provider_name: provider_name.into(),
            method_name: method_name.into(),
        }
    }

    /// Create an invalid configuration error
    pub fn invalid_configuration(provider_name: impl Into<String>, errors: Vec<String>) -> Self {
        Self::InvalidConfiguration {
            context: ErrorContext::new(format!("Provider '{}' has invalid configuration: {} error(s)", provider_name.into(), errors.len()))
                .with_kind(ErrorKind::Provider)
                .with_severity(ErrorSeverity::Error),
            provider_name: provider_name.into(),
            errors,
        }
    }

    /// Create a timeout error
    pub fn timeout(provider_name: impl Into<String>, operation: impl Into<String>, timeout_ms: u64) -> Self {
        Self::Timeout {
            context: ErrorContext::new(format!("Provider timeout: {} operation timed out after {}ms", provider_name.into(), timeout_ms))
                .with_kind(ErrorKind::Provider)
                .with_severity(ErrorSeverity::Error)
                .with_operation(operation),
            provider_name: provider_name.into(),
            operation: operation.into(),
            timeout_ms,
        }
    }

    /// Create an unavailable error
    pub fn unavailable(provider_name: impl Into<String>, reason: impl Into<String>) -> Self {
        Self::Unavailable {
            context: ErrorContext::new(format!("Provider '{}' unavailable: {}", provider_name.into(), reason.into()))
                .with_kind(ErrorKind::Provider)
                .with_severity(ErrorSeverity::Error),
            provider_name: provider_name.into(),
            reason: reason.into(),
        }
    }
}

impl fmt::Display for ProviderError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.context())
    }
}

impl StdError for ProviderError {}

impl From<ProviderError> for MountainError {
    fn from(err: ProviderError) -> Self {
        MountainError::new(err.context().clone())
            .with_source(err.to_string())
    }
}
