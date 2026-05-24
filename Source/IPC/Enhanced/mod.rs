//! # Enhanced IPC Features
//!
//! Advanced IPC enhancements for Mountain including:
//! - Message compression and batching
//! - Connection pooling and multiplexing
//! - Security enhancements
//! - Performance monitoring and distributed tracing
pub mod initialize_enhanced_ipc;
pub mod create_enhanced_handler;
pub mod integrate_with_tauri_ipc;
pub mod high_security_config;
pub mod high_performance_config;
pub mod get_statistics;
pub mod send_enhanced_message;
pub mod stop;
pub mod start;
pub mod new;

pub mod MessageCompressor;

pub mod ConnectionPool;

pub mod SecureMessageChannel;

pub mod PerformanceDashboard;

use std::collections::HashMap;

use bincode::serde::encode_to_vec;

// Import only the types, not the modules themselves (modules are already in scope via `pub mod`)
use crate::IPC::Enhanced::MessageCompressor::{
	BatchConfig::Struct as BatchConfig,
	CompressionAlgorithm::Enum as CompressionAlgorithm,
	CompressionLevel::Enum as CompressionLevel,
};
use crate::{
	IPC::Enhanced::{
		PerformanceDashboard::{
			DashboardConfig::Struct as DashboardConfig,
			DashboardStatistics::Struct as DashboardStatistics,
			MetricType::Enum as MetricType,
		},
		SecureMessageChannel::{
			EncryptedMessage::Struct as EncryptedMessage,
			SecurityConfig::Struct as SecurityConfig,
			SecurityStats::Struct as SecurityStats,
		},
		Struct::{PoolConfig::Struct as PoolConfig, PoolStats::Struct as PoolStats},
	},
	dev_log,
};

/// Enhanced IPC manager that combines all advanced features
pub struct EnhancedIPCManager {
	pub compressor:MessageCompressor::Struct::Struct,

	pub connection_pool:Struct::Pool::Struct,

	pub secure_channel:SecureMessageChannel::Channel::Struct,

	pub performance_dashboard:PerformanceDashboard::Dashboard::Struct,
}

/// Enhanced IPC statistics
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct EnhancedIPCStats {
	pub connection_pool:PoolStats,

	pub security:SecurityStats,

	pub performance:DashboardStatistics,

	pub compression_ratio:f64,
}

#[cfg(test)]
mod tests {

	use super::*;

	#[tokio::test]
	async fn test_enhanced_ipc_manager_creation() {
		let manager = EnhancedIPCManager::new().unwrap();

		assert!(manager.Start().await.is_ok());
		assert!(manager.Stop().await.is_ok());
	}

	#[tokio::test]
	async fn test_high_performance_config() {
		let manager = EnhancedIPCManager::high_performance_config();
		assert_eq!(manager.connection_pool.config.MaxConnections, 50);
	}

	#[tokio::test]
	async fn test_high_security_config() {
		let manager = EnhancedIPCManager::high_security_config();
		assert_eq!(manager.secure_channel.config.key_rotation_interval_hours, 1);
	}

	#[tokio::test]
	async fn test_statistics_collection() {
		let manager = EnhancedIPCManager::new().unwrap();
		manager.Start().await.unwrap();
		let stats = manager.GetStatistics().await;
		assert!(stats.compression_ratio >= 0.0);
		manager.Stop().await.unwrap();
	}
}

#[derive(Debug, Clone)]
pub struct Struct;
