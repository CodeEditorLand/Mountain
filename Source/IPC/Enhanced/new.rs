//! `Enhanced::new`

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

pub fn Fn() -> Result<Self, String> {
	let compressor_config = BatchConfig::default();

	let pool_config = PoolConfig::default();

	let security_config = SecurityConfig::default();

	let dashboard_config = DashboardConfig::default();

	Ok(Self {
		compressor:MessageCompressor::Struct::Struct::new(compressor_config),
		connection_pool:Struct::Pool::Struct::new(pool_config),
		secure_channel:SecureMessageChannel::Channel::Struct::new(security_config)?,
		performance_dashboard:PerformanceDashboard::Dashboard::Struct::new(dashboard_config),
	})
}
