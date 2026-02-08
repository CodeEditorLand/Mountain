//! # Configuration Error Types
//!
//! Provides configuration management error types for Mountain.
//! Used for all configuration related errors.

use super::CoreError::{ErrorContext, ErrorKind, ErrorSeverity, MountainError, Result};
use serde::{Deserialize, Serialize};
use std::error::Error as StdError;
use std::fmt;

/// Configuration operation error types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ConfigurationError {
    /// Configuration key not found
    KeyNotFound {
        context: ErrorContext,
        key: String,
        section: Option<String>,
    },
    /// Invalid configuration value
    InvalidValue {
        context: ErrorContext,
        key: String,
        expected_type: String,
    },
    /// Configuration validation failed
    ValidationFailed {
        context: ErrorContext,
        errors: Vec<String>,
    },
    /// Configuration parse error
    ParseError {
        context: ErrorContext,
        format: String,
        source: String,
    },
    /// Configuration file not found
    FileNotFound {
        context: ErrorContext,
        path: String,
    },
    /// Configuration file read error
    FileReadError {
        context: ErrorContext,
        path: String,
        source: String,
    },
    /// Configuration file write error
    FileWriteError {
        context: ErrorContext,
        path: String,
        source: String,
    },
    /// Circular dependency detected
    CircularDependency {
        context: ErrorContext,
        keys: Vec<String>,
    },
}

impl ConfigurationError {
    /// Get the error context
    pub fn context(&self) -> &ErrorContext {
        match self {
            ConfigurationError::KeyNotFound { context, .. } => context,
            ConfigurationError::InvalidValue { context, .. } => context,
            ConfigurationError::ValidationFailed { context, .. } => context,
            ConfigurationError::ParseError { context, .. } => context,
            ConfigurationError::FileNotFound { context, .. } => context,
            ConfigurationError::FileReadError { context, .. } => context,
            ConfigurationError::FileWriteError { context, .. } => context,
            ConfigurationError::CircularDependency { context, .. } => context,
        }
    }

    /// Create a key not found error
    pub fn key_not_found(key: impl Into<String>, section: Option<String>) -> Self {
        let key = key.into();
        let message = if let Some(section) = &section {
            format!("Configuration key '{}' not found in section '{}'", key, section)
        } else {
            format!("Configuration key '{}' not found", key)
        };
        Self::KeyNotFound {
            context: ErrorContext::new(message)
                .with_kind(ErrorKind::Configuration)
                .with_severity(ErrorSeverity::Error),
            key,
            section,
        }
    }

    /// Create an invalid value error
    pub fn invalid_value(key: impl Into<String>, expected_type: impl Into<String>) -> Self {
        Self::InvalidValue {
            context: ErrorContext::new(format!("Invalid value for key '{}': expected type '{}'", key.into(), expected_type.into()))
                .with_kind(ErrorKind::Configuration)
                .with_severity(ErrorSeverity::Error),
            key: key.into(),
            expected_type: expected_type.into(),
        }
    }

    /// Create a validation failed error
    pub fn validation_failed(errors: Vec<String>) -> Self {
        Self::ValidationFailed {
            context: ErrorContext::new(format!("Configuration validation failed with {} error(s)", errors.len()))
                .with_kind(ErrorKind::Configuration)
                .with_severity(ErrorSeverity::Error),
            errors,
        }
    }

    /// Create a parse error
    pub fn parse_error(format: impl Into<String>, source: impl Into<String>, message: impl Into<String>) -> Self {
        Self::ParseError {
            context: ErrorContext::new(message)
                .with_kind(ErrorKind::Configuration)
                .with_severity(ErrorSeverity::Error),
            format: format.into(),
            source: source.into(),
        }
    }

    /// Create a file not found error
    pub fn file_not_found(path: impl Into<String>) -> Self {
        Self::FileNotFound {
            context: ErrorContext::new(format!("Configuration file not found: {}", path.into()))
                .with_kind(ErrorKind::Configuration)
                .with_severity(ErrorSeverity::Error),
            path: path.into(),
        }
    }

    /// Create a circular dependency error
    pub fn circular_dependency(keys: Vec<String>) -> Self {
        Self::CircularDependency {
            context: ErrorContext::new(format!("Circular dependency detected in configuration: {}", keys.join(" -> ")))
                .with_kind(ErrorKind::Configuration)
                .with_severity(ErrorSeverity::Critical),
            keys,
        }
    }
}

impl fmt::Display for ConfigurationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.context())
    }
}

impl StdError for ConfigurationError {}

impl From<ConfigurationError> for MountainError {
    fn from(err: ConfigurationError) -> Self {
        MountainError::new(err.context().clone())
            .with_source(err.to_string())
    }
}
