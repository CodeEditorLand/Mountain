//! # Connection Pooling and Multiplexing
//!
//! Advanced connection pooling for concurrent IPC operations with health
//! monitoring and connection lifecycle management.

use std::{
	collections::HashMap,
	sync::Arc,
	time::{Duration, Instant},
};

use log::{debug, error, info, trace, warn};
use serde::{Deserialize, Serialize};
use tokio::{
	sync::{Mutex as AsyncMutex, Notify, RwLock, Semaphore},
	time::{interval, timeout},
};
use uuid::Uuid;

/// Connection pool configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PoolConfig {
	pub max_connections:usize,
	pub min_connections:usize,
	pub connection_timeout_ms:u64,
	pub max_lifetime_ms:u64,
	pub idle_timeout_ms:u64,
	pub health_check_interval_ms:u64,
}

impl Default for PoolConfig {
	fn default() -> Self {
		Self {
			max_connections:10,
			min_connections:2,
			connection_timeout_ms:30000,    // 30 seconds
			max_lifetime_ms:300000,         // 5 minutes
			idle_timeout_ms:60000,          // 1 minute
			health_check_interval_ms:30000, // 30 seconds
		}
	}
}

/// Connection handle with health monitoring
#[derive(Debug, Clone)]
pub struct ConnectionHandle {
	pub id:String,
	pub created_at:Instant,
	pub last_used:Instant,
	pub health_score:f64,
	pub error_count:usize,
	pub successful_operations:usize,
	pub total_operations:usize,
	pub is_active:bool,
}

impl ConnectionHandle {
	/// Create a new connection handle
	pub fn new() -> Self {
		Self {
			id:Uuid::new_v4().to_string(),
			created_at:Instant::now(),
			last_used:Instant::now(),
			health_score:100.0,
			error_count:0,
			successful_operations:0,
			total_operations:0,
			is_active:true,
		}
	}

	/// Update health based on operation success
	pub fn update_health(&mut self, success:bool) {
		self.last_used = Instant::now();
		self.total_operations += 1;

		if success {
			self.successful_operations += 1;
			// Increase health score gradually
			self.health_score = (self.health_score + 2.0).min(100.0);
			self.error_count = 0;
		} else {
			self.error_count += 1;
			// Decrease health score more aggressively
			self.health_score = (self.health_score - 10.0).max(0.0);
		}

		// Calculate success rate
		let success_rate = if self.total_operations > 0 {
			self.successful_operations as f64 / self.total_operations as f64
		} else {
			1.0
		};

		// Adjust health score based on overall success rate
		self.health_score = (self.health_score * 0.7 + success_rate * 100.0 * 0.3).max(0.0).min(100.0);
	}

	/// Check if connection is healthy
	pub fn is_healthy(&self) -> bool {
		self.health_score > 50.0 && self.error_count < 5 && self.is_active && self.age().as_secs() < 300 // Less than 5 minutes old
	}

	/// Get connection age
	pub fn age(&self) -> Duration { self.created_at.elapsed() }

	/// Get idle time
	pub fn idle_time(&self) -> Duration { self.last_used.elapsed() }

	/// Get success rate
	pub fn success_rate(&self) -> f64 {
		if self.total_operations == 0 {
			1.0
		} else {
			self.successful_operations as f64 / self.total_operations as f64
		}
	}
}

/// Connection pool statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PoolStats {
	pub total_connections:usize,
	pub active_connections:usize,
	pub idle_connections:usize,
	pub healthy_connections:usize,
	pub max_connections:usize,
	pub min_connections:usize,
	pub wait_queue_size:usize,
	pub average_wait_time_ms:f64,
	pub total_operations:u64,
	pub successful_operations:u64,
	pub error_rate:f64,
}

/// Connection pool with advanced management
pub struct ConnectionPool {
	pub config:PoolConfig,
	pub connections:Arc<AsyncMutex<HashMap<String, ConnectionHandle>>>,
	pub semaphore:Arc<Semaphore>,
	pub wait_queue:Arc<AsyncMutex<Vec<Arc<Notify>>>>,
	pub stats:Arc<RwLock<PoolStats>>,
	pub health_checker:Arc<AsyncMutex<ConnectionHealthChecker>>,
	pub is_running:Arc<AsyncMutex<bool>>,
}

impl ConnectionPool {
	/// Create a new connection pool
	pub fn new(config:PoolConfig) -> Self {
		let pool = Self {
			config,
			connections:Arc::new(AsyncMutex::new(HashMap::new())),
			semaphore:Arc::new(Semaphore::new(config.max_connections)),
			wait_queue:Arc::new(AsyncMutex::new(Vec::new())),
			stats:Arc::new(RwLock::new(PoolStats {
				total_connections:0,
				active_connections:0,
				idle_connections:0,
				healthy_connections:0,
				max_connections:config.max_connections,
				min_connections:config.min_connections,
				wait_queue_size:0,
				average_wait_time_ms:0.0,
				total_operations:0,
				successful_operations:0,
				error_rate:0.0,
			})),
			health_checker:Arc::new(AsyncMutex::new(ConnectionHealthChecker::new())),
			is_running:Arc::new(AsyncMutex::new(false)),
		};

		info!("[ConnectionPool] Created pool with max {} connections", config.max_connections);
		pool
	}

	/// Start the connection pool
	pub async fn start(&self) -> Result<(), String> {
		{
			let mut running = self.is_running.lock().await;
			if *running {
				return Ok(()); // Already running
			}
			*running = true;
		}

		// Start health monitoring
		self.start_health_monitoring().await;

		// Start connection cleanup
		self.start_connection_cleanup().await;

		// Initialize minimum connections
		self.initialize_min_connections().await;

		info!("[ConnectionPool] Started connection pool");
		Ok(())
	}

	/// Stop the connection pool
	pub async fn stop(&self) -> Result<(), String> {
		{
			let mut running = self.is_running.lock().await;
			if !*running {
				return Ok(()); // Already stopped
			}
			*running = false;
		}

		// Clear all connections
		{
			let mut connections = self.connections.lock().await;
			connections.clear();
		}

		// Notify all waiting tasks
		{
			let mut wait_queue = self.wait_queue.lock().await;
			for notifier in wait_queue.drain(..) {
				notifier.notify_one();
			}
		}

		info!("[ConnectionPool] Stopped connection pool");
		Ok(())
	}

	/// Get a connection from the pool
	pub async fn get_connection(&self) -> Result<ConnectionHandle, String> {
		let start_time = Instant::now();

		// Try to acquire permit with timeout
		let permit = timeout(
			Duration::from_millis(self.config.connection_timeout_ms),
			self.semaphore.acquire(),
		)
		.await
		.map_err(|_| "Connection timeout".to_string())?
		.map_err(|e| format!("Failed to acquire connection: {}", e))?;

		let wait_time = start_time.elapsed().as_millis() as f64;

		// Update wait time statistics
		{
			let mut stats = self.stats.write().await;
			stats.average_wait_time_ms = (stats.average_wait_time_ms * stats.total_operations as f64 + wait_time)
				/ (stats.total_operations as f64 + 1.0);
		}

		// Find or create a healthy connection
		let connection = self.find_or_create_connection().await?;

		// Update statistics
		{
			let mut stats = self.stats.write().await;
			stats.active_connections += 1;
			stats.total_operations += 1;
		}

		trace!("[ConnectionPool] Connection acquired: {}", connection.id);
		Ok(connection)
	}

	/// Release a connection back to the pool
	pub async fn release_connection(&self, mut handle:ConnectionHandle) {
		handle.last_used = Instant::now();

		// Update connection in pool
		{
			let mut connections = self.connections.lock().await;
			connections.insert(handle.id.clone(), handle.clone());
		}

		// Update statistics
		{
			let mut stats = self.stats.write().await;
			stats.active_connections = stats.active_connections.saturating_sub(1);
			stats.idle_connections += 1;
		}

		// Release permit
		drop(handle); // The permit is released when the handle is dropped

		trace!("[ConnectionPool] Connection released: {}", handle.id);
	}

	/// Find or create a healthy connection
	async fn find_or_create_connection(&self) -> Result<ConnectionHandle, String> {
		let mut connections = self.connections.lock().await;

		// Try to find a healthy connection
		for (id, handle) in connections.iter_mut() {
			if handle.is_healthy() && handle.idle_time().as_millis() < self.config.idle_timeout_ms as u128 {
				handle.last_used = Instant::now();
				return Ok(handle.clone());
			}
		}

		// No healthy connection found, create new one
		let new_handle = ConnectionHandle::new();
		connections.insert(new_handle.id.clone(), new_handle.clone());

		// Update statistics
		{
			let mut stats = self.stats.write().await;
			stats.total_connections += 1;
			stats.healthy_connections += 1;
		}

		Ok(new_handle)
	}

	/// Start health monitoring
	async fn start_health_monitoring(&self) {
		let pool = Arc::new(self.clone());

		tokio::spawn(async move {
			let mut interval = interval(Duration::from_millis(pool.config.health_check_interval_ms));

			while *pool.is_running.lock().await {
				interval.tick().await;

				if let Err(e) = pool.check_connection_health().await {
					error!("[ConnectionPool] Health check failed: {}", e);
				}
			}
		});
	}

	/// Start connection cleanup
	async fn start_connection_cleanup(&self) {
		let pool = Arc::new(self.clone());

		tokio::spawn(async move {
			let mut interval = interval(Duration::from_secs(60)); // Check every minute

			while *pool.is_running.lock().await {
				interval.tick().await;

				let cleaned_count = pool.cleanup_stale_connections().await;
				if cleaned_count > 0 {
					debug!("[ConnectionPool] Cleaned {} stale connections", cleaned_count);
				}
			}
		});
	}

	/// Initialize minimum connections
	async fn initialize_min_connections(&self) {
		let current_count = self.connections.lock().await.len();

		if current_count < self.config.min_connections {
			let needed = self.config.min_connections - current_count;

			for _ in 0..needed {
				let handle = ConnectionHandle::new();
				let mut connections = self.connections.lock().await;
				connections.insert(handle.id.clone(), handle);
			}

			debug!("[ConnectionPool] Initialized {} minimum connections", needed);
		}
	}

	/// Check connection health
	async fn check_connection_health(&self) -> Result<(), String> {
		let mut connections = self.connections.lock().await;
		let mut health_checker = self.health_checker.lock().await;

		let mut healthy_count = 0;

		for (id, handle) in connections.iter_mut() {
			let is_healthy = health_checker.check_connection_health(handle).await;
			handle.update_health(is_healthy);

			if handle.is_healthy() {
				healthy_count += 1;
			}
		}

		// Update statistics
		{
			let mut stats = self.stats.write().await;
			stats.healthy_connections = healthy_count;
			stats.idle_connections = connections.len().saturating_sub(stats.active_connections);

			if stats.total_operations > 0 {
				stats.error_rate = 1.0 - (stats.successful_operations as f64 / stats.total_operations as f64);
			}
		}

		Ok(())
	}

	/// Cleanup stale connections
	async fn cleanup_stale_connections(&self) -> usize {
		let mut connections = self.connections.lock().await;
		let now = Instant::now();

		let stale_ids:Vec<String> = connections
			.iter()
			.filter(|(_, handle)| {
				handle.age().as_millis() > self.config.max_lifetime_ms as u128
					|| handle.idle_time().as_millis() > self.config.idle_timeout_ms as u128
					|| !handle.is_healthy()
			})
			.map(|(id, _)| id.clone())
			.collect();

		for id in &stale_ids {
			connections.remove(id);
		}

		// Update statistics
		{
			let mut stats = self.stats.write().await;
			stats.total_connections = connections.len();
			stats.healthy_connections = connections.values().filter(|h| h.is_healthy()).count();
		}

		stale_ids.len()
	}

	/// Get pool statistics
	pub async fn get_stats(&self) -> PoolStats { self.stats.read().await.clone() }

	/// Get active connection count
	pub async fn get_active_count(&self) -> usize { self.stats.read().await.active_connections }

	/// Get healthy connection count
	pub async fn get_healthy_count(&self) -> usize { self.stats.read().await.healthy_connections }

	/// Check if pool is running
	pub async fn is_running(&self) -> bool { *self.is_running.lock().await }
}

impl Clone for ConnectionPool {
	fn clone(&self) -> Self {
		Self {
			config:self.config.clone(),
			connections:self.connections.clone(),
			semaphore:self.semaphore.clone(),
			wait_queue:self.wait_queue.clone(),
			stats:self.stats.clone(),
			health_checker:self.health_checker.clone(),
			is_running:self.is_running.clone(),
		}
	}
}

/// Connection health checker
struct ConnectionHealthChecker {
	ping_timeout:Duration,
}

impl ConnectionHealthChecker {
	fn new() -> Self { Self { ping_timeout:Duration::from_secs(5) } }

	/// Check connection health
	async fn check_connection_health(&self, handle:&mut ConnectionHandle) -> bool {
		// Simulate health check by ensuring connection can handle basic operations
		// In a real implementation, this would send an actual ping message
		let start_time = Instant::now();

		// Simulate network latency
		tokio::time::sleep(Duration::from_millis(10)).await;

		let response_time = start_time.elapsed();

		// Connection is healthy if response time is reasonable
		response_time < self.ping_timeout
	}
}

/// Utility functions for connection pooling
impl ConnectionPool {
	/// Create a connection pool with default configuration
	pub fn default_pool() -> Self { Self::new(PoolConfig::default()) }

	/// Create a high-performance pool
	pub fn high_performance_pool() -> Self {
		Self::new(PoolConfig {
			max_connections:50,
			min_connections:10,
			connection_timeout_ms:10000,
			max_lifetime_ms:180000,         // 3 minutes
			idle_timeout_ms:30000,          // 30 seconds
			health_check_interval_ms:15000, // 15 seconds
		})
	}

	/// Create a conservative pool
	pub fn conservative_pool() -> Self {
		Self::new(PoolConfig {
			max_connections:5,
			min_connections:1,
			connection_timeout_ms:60000,
			max_lifetime_ms:600000,         // 10 minutes
			idle_timeout_ms:120000,         // 2 minutes
			health_check_interval_ms:60000, // 60 seconds
		})
	}

	/// Calculate optimal pool size based on system resources
	pub fn calculate_optimal_pool_size() -> usize {
		let num_cpus = num_cpus::get();
		// Use 2x CPU count as optimal pool size
		(num_cpus * 2).max(4).min(50)
	}
}

#[cfg(test)]
mod tests {
	use tokio::time::sleep;

	use super::*;

	#[tokio::test]
	async fn test_connection_pool_creation() {
		let pool = ConnectionPool::default_pool();
		assert_eq!(pool.config.max_connections, 10);
	}

	#[tokio::test]
	async fn test_connection_handle_health() {
		let mut handle = ConnectionHandle::new();
		assert!(handle.is_healthy());

		// Test successful operation
		handle.update_health(true);
		assert!(handle.is_healthy());
		assert_eq!(handle.success_rate(), 1.0);

		// Test failed operation
		handle.update_health(false);
		assert!(handle.is_healthy()); // Should still be healthy after one failure
		assert_eq!(handle.success_rate(), 0.5);
	}

	#[tokio::test]
	async fn test_pool_lifecycle() {
		let pool = ConnectionPool::default_pool();

		// Start pool
		pool.start().await.unwrap();
		assert!(pool.is_running().await);

		// Get connection
		let handle = pool.get_connection().await.unwrap();
		assert!(handle.is_healthy());

		// Release connection
		pool.release_connection(handle).await;

		// Stop pool
		pool.stop().await.unwrap();
		assert!(!pool.is_running().await);
	}

	#[tokio::test]
	async fn test_pool_statistics() {
		let pool = ConnectionPool::default_pool();
		pool.start().await.unwrap();

		// Get some connections
		let handles:Vec<ConnectionHandle> = (0..3).map(|_| pool.get_connection().await.unwrap()).collect();

		let stats = pool.get_stats().await;
		assert_eq!(stats.active_connections, 3);

		// Release connections
		for handle in handles {
			pool.release_connection(handle).await;
		}

		let stats_after = pool.get_stats().await;
		assert_eq!(stats_after.active_connections, 0);
		assert_eq!(stats_after.idle_connections, 3);

		pool.stop().await.unwrap();
	}

	#[tokio::test]
	async fn test_connection_cleanup() {
		let pool = ConnectionPool::new(PoolConfig {
			max_lifetime_ms:100, // Very short lifetime for testing
			idle_timeout_ms:50,
			..Default::default()
		});

		pool.start().await.unwrap();

		// Get and release connection
		let handle = pool.get_connection().await.unwrap();
		pool.release_connection(handle).await;

		// Wait for cleanup
		sleep(Duration::from_millis(200)).await;

		let cleaned_count = pool.cleanup_stale_connections().await;
		assert!(cleaned_count > 0);

		pool.stop().await.unwrap();
	}

	#[test]
	fn test_optimal_pool_size_calculation() {
		let optimal_size = ConnectionPool::calculate_optimal_pool_size();
		assert!(optimal_size >= 4 && optimal_size <= 50);
	}
}
