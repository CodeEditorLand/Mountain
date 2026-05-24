//! `Enhanced::initialize_enhanced_ipc`

use std::collections::HashMap;

use bincode::serde::encode_to_vec;

use super::Struct;
use crate::{
	IPC::Enhanced::{
		MessageCompressor::{
			BatchConfig::Struct as BatchConfig,
			CompressionAlgorithm::Enum as CompressionAlgorithm,
			CompressionLevel::Enum as CompressionLevel,
		},
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

/// Initialize enhanced IPC features
pub fn Fn() -> Result<EnhancedIPCManager, String> {
	let manager = EnhancedIPCManager::new()?;

	manager.Start().await?;

	dev_log!("ipc", "[EnhancedIPCManager] Enhanced IPC features initialized");

	Ok(manager)
}
