//! `Enhanced::high_security_config`

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

pub fn Fn() -> Struct {
	let compressor_config = BatchConfig {
		MaxBatchSize:50,

		MaxBatchDelayMs:200,

		CompressionThresholdBytes:2048,

		CompressionLevel:CompressionLevel::Balanced,

		Algorithm:CompressionAlgorithm::Gzip,
	};

	let pool_config = PoolConfig {
		max_connections:10,

		min_connections:2,

		connection_timeout_ms:30000,

		max_lifetime_ms:600000,

		idle_timeout_ms:120000,

		health_check_interval_ms:60000,
	};

	let security_config = SecurityConfig {
		key_rotation_interval_hours:1,

		max_message_size_bytes:1 * 1024 * 1024,
		..Default::default()
	};

	let dashboard_config = DashboardConfig {
		update_interval_ms:2000,

		metrics_retention_hours:48,

		alert_threshold_ms:2000,

		trace_sampling_rate:0.5,

		max_traces_stored:500,
	};

	Self {
		compressor:MessageCompressor::Struct::Struct::new(compressor_config),

		connection_pool:Struct::Pool::Struct::new(pool_config),

		secure_channel:SecureMessageChannel::Channel::Struct::new(security_config).unwrap(),

		performance_dashboard:PerformanceDashboard::Dashboard::Struct::new(dashboard_config),
	}
}
