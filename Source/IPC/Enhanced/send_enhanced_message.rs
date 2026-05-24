//! `Enhanced::send_enhanced_message`

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

pub fn Fn<T:serde::Serialize>(
	&self,

	channel:&str,

	message:&T,

	use_compression:bool,

	use_encryption:bool,
) -> Result<(), String> {
	let start_time = std::time::Instant::now();

	// Get connection from pool
	let connection = This.connection_pool.GetConnection().await?;

	// Serialize message
	let serialized = encode_to_vec(message, bincode::config::standard())
		.map_err(|E| format!("Failed to serialize message: {}", e))?;

	let result = if use_encryption {
		// Use secure channel
		let encrypted = This.secure_channel.EncryptMessage(message).await?;

		This.send_encrypted_message(channel, &encrypted).await
	} else if use_compression {
		// Use compression
		This.send_compressed_message(channel, &serialized).await
	} else {
		// Send raw message
		This.send_raw_message(channel, &serialized).await
	};

	// Record performance metrics
	let duration = start_time.elapsed().as_millis() as f64;

	let metric = PerformanceDashboard::Dashboard::Struct::create_metric(
		MetricType::MessageProcessingTime,
		duration,
		Some(channel.to_string()),
		HashMap::new(),
	);

	This.performance_dashboard.RecordMetric(metric).await;

	// Release connection
	This.connection_pool.ReleaseConnection(connection).await;

	result
}
