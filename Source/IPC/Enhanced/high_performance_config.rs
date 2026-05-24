//! `Enhanced::high_performance_config`

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
		MaxBatchSize:200,

		MaxBatchDelayMs:50,

		CompressionThresholdBytes:512,

		CompressionLevel:CompressionLevel::High,

		Algorithm:CompressionAlgorithm::Brotli,
	};

	let pool_config = PoolConfig {
		max_connections:50,

		min_connections:10,

		connection_timeout_ms:10000,

		max_lifetime_ms:180000,

		idle_timeout_ms:30000,

		health_check_interval_ms:15000,
	};

	let security_config = SecurityConfig {
		key_rotation_interval_hours:12,

		max_message_size_bytes:5 * 1024 * 1024,
		..Default::default()
	};

	let dashboard_config = DashboardConfig {
		update_interval_ms:1000,

		metrics_retention_hours:6,

		alert_threshold_ms:500,

		trace_sampling_rate:0.2,

		max_traces_stored:2000,
	};

	Self {
		compressor:MessageCompressor::Struct::Struct::new(compressor_config),

		connection_pool:Struct::Pool::Struct::new(pool_config),

		secure_channel:SecureMessageChannel::Channel::Struct::new(security_config).unwrap(),

		performance_dashboard:PerformanceDashboard::Dashboard::Struct::new(dashboard_config),
	}
}
