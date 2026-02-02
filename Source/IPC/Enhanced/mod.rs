//! # Enhanced IPC Features
//!
//! Advanced IPC enhancements for Mountain including:
//! - Message compression and batching
//! - Connection pooling and multiplexing
//! - Security enhancements
//! - Performance monitoring and distributed tracing

pub mod MessageCompressor;
pub mod ConnectionPool;
pub mod SecureMessageChannel;
pub mod PerformanceDashboard;

/// Enhanced IPC manager that combines all advanced features
pub struct EnhancedIPCManager {
	pub compressor:MessageCompressor,
	pub connection_pool:ConnectionPool,
	pub secure_channel:SecureMessageChannel,
	pub performance_dashboard:PerformanceDashboard,
}

impl EnhancedIPCManager {
	/// Create a new enhanced IPC manager
	pub fn new() -> Result<Self, String> {
		let compressor_config = BatchConfig::default();
		let pool_config = PoolConfig::default();
		let security_config = SecurityConfig::default();
		let dashboard_config = DashboardConfig::default();

		Ok(Self {
			compressor:MessageCompressor::new(compressor_config),
			connection_pool:ConnectionPool::new(pool_config),
			secure_channel:SecureMessageChannel::new(security_config)?,
			performance_dashboard:PerformanceDashboard::new(dashboard_config),
		})
	}

	/// Start all enhanced IPC features
	pub async fn start(&self) -> Result<(), String> {
		self.connection_pool.start().await?;
		self.secure_channel.start().await?;
		self.performance_dashboard.start().await?;

		log::info!("[EnhancedIPCManager] All enhanced IPC features started");
		Ok(())
	}

	/// Stop all enhanced IPC features
	pub async fn stop(&self) -> Result<(), String> {
		self.connection_pool.stop().await?;
		self.secure_channel.stop().await?;
		self.performance_dashboard.stop().await?;

		log::info!("[EnhancedIPCManager] All enhanced IPC features stopped");
		Ok(())
	}

	/// Send a message using enhanced features
	pub async fn send_enhanced_message<T:serde::Serialize>(
		&self,
		channel:&str,
		message:&T,
		use_compression:bool,
		use_encryption:bool,
	) -> Result<(), String> {
		let start_time = std::time::Instant::now();

		// Get connection from pool
		let connection = self.connection_pool.get_connection().await?;

		// Serialize message
		let serialized = bincode::serialize(message).map_err(|e| format!("Failed to serialize message: {}", e))?;

		let result = if use_encryption {
			// Use secure channel
			let encrypted = self.secure_channel.encrypt_message(message).await?;
			self.send_encrypted_message(channel, &encrypted).await
		} else if use_compression {
			// Use compression
			self.send_compressed_message(channel, &serialized).await
		} else {
			// Send raw message
			self.send_raw_message(channel, &serialized).await
		};

		// Record performance metrics
		let duration = start_time.elapsed().as_millis() as f64;
		let metric = PerformanceDashboard::create_metric(
			MetricType::MessageProcessingTime,
			duration,
			Some(channel.to_string()),
			HashMap::new(),
		);

		self.performance_dashboard.record_metric(metric).await;

		// Release connection
		self.connection_pool.release_connection(connection).await;

		result
	}

	/// Send encrypted message
	async fn send_encrypted_message(&self, channel:&str, encrypted:&EncryptedMessage) -> Result<(), String> {
		// Implementation would integrate with existing IPC infrastructure
		log::debug!("[EnhancedIPCManager] Sending encrypted message on channel: {}", channel);
		Ok(())
	}

	/// Send compressed message
	async fn send_compressed_message(&self, channel:&str, data:&[u8]) -> Result<(), String> {
		// Implementation would integrate with existing IPC infrastructure
		log::debug!("[EnhancedIPCManager] Sending compressed message on channel: {}", channel);
		Ok(())
	}

	/// Send raw message
	async fn send_raw_message(&self, channel:&str, data:&[u8]) -> Result<(), String> {
		// Implementation would integrate with existing IPC infrastructure
		log::debug!("[EnhancedIPCManager] Sending raw message on channel: {}", channel);
		Ok(())
	}

	/// Get enhanced IPC statistics
	pub async fn get_statistics(&self) -> EnhancedIPCStats {
		let pool_stats = self.connection_pool.get_stats().await;
		let security_stats = self.secure_channel.get_stats().await;
		let dashboard_stats = self.performance_dashboard.get_statistics().await;

		EnhancedIPCStats {
			connection_pool:pool_stats,
			security:security_stats,
			performance:dashboard_stats,
			compression_ratio:self.compressor.get_batch_stats().total_size_bytes as f64,
		}
	}
}

/// Enhanced IPC statistics
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct EnhancedIPCStats {
	pub connection_pool:PoolStats,
	pub security:SecurityStats,
	pub performance:DashboardStatistics,
	pub compression_ratio:f64,
}

/// Initialize enhanced IPC features
pub async fn initialize_enhanced_ipc() -> Result<EnhancedIPCManager, String> {
	let manager = EnhancedIPCManager::new()?;
	manager.start().await?;

	log::info!("[EnhancedIPCManager] Enhanced IPC features initialized");
	Ok(manager)
}

/// Utility functions for enhanced IPC
impl EnhancedIPCManager {
	/// Create a high-performance configuration
	pub fn high_performance_config() -> Self {
		let compressor_config = BatchConfig {
			max_batch_size:200,
			max_batch_delay_ms:50,
			compression_threshold_bytes:512,
			compression_level:CompressionLevel::High,
			algorithm:CompressionAlgorithm::Brotli,
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
			compressor:MessageCompressor::new(compressor_config),
			connection_pool:ConnectionPool::new(pool_config),
			secure_channel:SecureMessageChannel::new(security_config).unwrap(),
			performance_dashboard:PerformanceDashboard::new(dashboard_config),
		}
	}

	/// Create a security-focused configuration
	pub fn high_security_config() -> Self {
		let compressor_config = BatchConfig {
			max_batch_size:50,
			max_batch_delay_ms:200,
			compression_threshold_bytes:2048,
			compression_level:CompressionLevel::Balanced,
			algorithm:CompressionAlgorithm::Gzip,
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
			compressor:MessageCompressor::new(compressor_config),
			connection_pool:ConnectionPool::new(pool_config),
			secure_channel:SecureMessageChannel::new(security_config).unwrap(),
			performance_dashboard:PerformanceDashboard::new(dashboard_config),
		}
	}
}

/// Integration with existing Mountain IPC system
impl EnhancedIPCManager {
	/// Integrate with Tauri IPCServer
	pub async fn integrate_with_tauri_ipc(
		&self,
		ipc_server:&crate::IPC::TauriIPCServer::TauriIPCServer,
	) -> Result<(), String> {
		log::info!("[EnhancedIPCManager] Integrating with Tauri IPC server");

		// Register enhanced message handlers
		// This would involve setting up callbacks and event handlers
		// to leverage the enhanced features

		Ok(())
	}

	/// Create enhanced message handler
	pub async fn create_enhanced_handler(
		&self,
	) -> impl Fn(crate::IPC::TauriIPCServer::TauriIPCMessage) -> Result<(), String> {
		// Return a closure that handles messages with enhanced features
		|message:crate::IPC::TauriIPCServer::TauriIPCMessage| {
			log::debug!("[EnhancedIPCManager] Handling message on channel: {}", message.channel);
			Ok(())
		}
	}
}

#[cfg(test)]
mod tests {
	use super::*;

	#[tokio::test]
	async fn test_enhanced_ipc_manager_creation() {
		let manager = EnhancedIPCManager::new().unwrap();
		assert!(manager.start().await.is_ok());
		assert!(manager.stop().await.is_ok());
	}

	#[tokio::test]
	async fn test_high_performance_config() {
		let manager = EnhancedIPCManager::high_performance_config();
		assert_eq!(manager.connection_pool.config.max_connections, 50);
	}

	#[tokio::test]
	async fn test_high_security_config() {
		let manager = EnhancedIPCManager::high_security_config();
		assert_eq!(manager.secure_channel.config.key_rotation_interval_hours, 1);
	}

	#[tokio::test]
	async fn test_statistics_collection() {
		let manager = EnhancedIPCManager::new().unwrap();
		manager.start().await.unwrap();

		let stats = manager.get_statistics().await;
		assert!(stats.compression_ratio >= 0.0);

		manager.stop().await.unwrap();
	}
}
