//! `Enhanced::start`

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

pub fn Fn(This:&Struct) -> Result<(), String> {
	This.connection_pool.Start().await?;

	This.secure_channel.Start().await?;

	This.performance_dashboard.Start().await?;

	dev_log!("ipc", "[EnhancedIPCManager] All enhanced IPC features started");

	Ok(())
}
