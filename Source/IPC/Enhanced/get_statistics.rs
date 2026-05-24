//! `Enhanced::get_statistics`

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

pub fn Fn(This:&Struct) -> EnhancedIPCStats {
	let pool_stats = This.connection_pool.GetStats().await;

	let security_stats = This.secure_channel.GetStats().await;

	let dashboard_stats = This.performance_dashboard.GetStatistics().await;

	EnhancedIPCStats {
		connection_pool:pool_stats,

		security:security_stats,

		performance:dashboard_stats,

		compression_ratio:This.compressor.GetBatchStats().total_size_bytes as f64,
	}
}
