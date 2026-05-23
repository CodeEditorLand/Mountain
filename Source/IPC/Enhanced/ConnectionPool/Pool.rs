//! `Pool::Struct` - bounded connection pool with health
//! monitoring, idle/lifetime cleanup, wait-queue timeouts, and
//! statistics. Acquire via `get_connection` (drops a permit on
//! the inner `Semaphore`); return via `release_connection`.
//! The struct + 18-method impl + Clone + tests stay in one
//! file - tightly coupled cluster.

use std::{
	collections::HashMap,
	sync::Arc,
	time::{Duration, Instant},
};

use tokio::{
	sync::{Mutex as AsyncMutex, Notify, RwLock, Semaphore},
	time::{interval, timeout},
};

use crate::{
	IPC::Enhanced::ConnectionPool::{
		ConnectionHandle::Struct as ConnectionHandle,
		HealthChecker::Struct as HealthChecker,
		PoolConfig::Struct as PoolConfig,
		PoolStats::Struct as PoolStats,
	},
	dev_log,
};

pub struct Struct {
	pub config:PoolConfig,

	pub connections:Arc<AsyncMutex<HashMap<String, ConnectionHandle>>>,

	pub semaphore:Arc<Semaphore>,

	pub wait_queue:Arc<AsyncMutex<Vec<Arc<Notify>>>>,

	pub stats:Arc<RwLock<PoolStats>>,

	pub health_checker:Arc<AsyncMutex<HealthChecker>>,

	pub is_running:Arc<AsyncMutex<bool>>,
}

impl Struct {
	pub fn new(config:PoolConfig) -> Self {
		let max_connections = config.max_connections;

		let min_connections = config.min_connections;

		let pool = Self {
			config:config.clone(),

			connections:Arc::new(AsyncMutex::new(HashMap::new())),

			semaphore:Arc::new(Semaphore::new(max_connections)),

			wait_queue:Arc::new(AsyncMutex::new(Vec::new())),

			stats:Arc::new(RwLock::new(PoolStats {
				total_connections:0,
				active_connections:0,
				idle_connections:0,
				healthy_connections:0,
				max_connections,
				min_connections,
				wait_queue_size:0,
				average_wait_time_ms:0.0,
				total_operations:0,
				successful_operations:0,
				error_rate:0.0,
			})),

			health_checker:Arc::new(AsyncMutex::new(HealthChecker::new())),

			is_running:Arc::new(AsyncMutex::new(false)),
		};

		dev_log!("ipc", "[ConnectionPool] Created pool with max {} connections", max_connections);

		pool
	}

	pub async fn start(&self) -> Result<(), String> {
		{
			let mut running = self.is_running.lock().await;

			if *running {
				return Ok(());
			}

			*running = true;
		}

		self.start_health_monitoring().await;

		self.start_connection_cleanup().await;

		self.initialize_min_connections().await;

		dev_log!("ipc", "[ConnectionPool] Started connection pool");

		Ok(())
	}

	pub async fn stop(&self) -> Result<(), String> {
		{
			let mut running = self.is_running.lock().await;

			if !*running {
				return Ok(());
			}

			*running = false;
		}

		{
			let mut connections = self.connections.lock().await;

			connections.clear();
		}

		{
			let mut wait_queue = self.wait_queue.lock().await;

			for notifier in wait_queue.drain(..) {
				notifier.notify_one();
			}
		}

		dev_log!("ipc", "[ConnectionPool] Stopped connection pool");

		Ok(())
	}

	pub async fn get_connection(&self) -> Result<ConnectionHandle, String> {
		let start_time = Instant::now();

		let _permit = timeout(
			Duration::from_millis(self.config.connection_timeout_ms),
			self.semaphore.acquire(),
		)
		.await
		.map_err(|_| "Connection timeout".to_string())?
		.map_err(|e| format!("Failed to acquire connection: {}", e))?;

		let wait_time = start_time.elapsed().as_millis() as f64;

		{
			let mut stats = self.stats.write().await;

			stats.average_wait_time_ms = (stats.average_wait_time_ms * stats.total_operations as f64 + wait_time)
				/ (stats.total_operations as f64 + 1.0);
		}

		let connection = self.find_or_create_connection().await?;

		{
			let mut stats = self.stats.write().await;

			stats.active_connections += 1;

			stats.total_operations += 1;
		}

		dev_log!("ipc", "[ConnectionPool] Connection acquired: {}", connection.id);

		Ok(connection)
	}

	pub async fn release_connection(&self, mut handle:ConnectionHandle) {
		let connection_id = handle.id.clone();

		handle.last_used = Instant::now();

		{
			let mut connections = self.connections.lock().await;

			connections.insert(handle.id.clone(), handle.clone());
		}

		{
			let mut stats = self.stats.write().await;

			stats.active_connections = stats.active_connections.saturating_sub(1);

			stats.idle_connections += 1;
		}

		drop(handle);

		dev_log!("ipc", "[ConnectionPool] Connection released: {}", connection_id);
	}

	async fn find_or_create_connection(&self) -> Result<ConnectionHandle, String> {
		let mut connections = self.connections.lock().await;

		for (_id, handle) in connections.iter_mut() {
			if handle.is_healthy() && handle.idle_time().as_millis() < self.config.idle_timeout_ms as u128 {
				handle.last_used = Instant::now();

				return Ok(handle.clone());
			}
		}

		let new_handle = ConnectionHandle::new();

		connections.insert(new_handle.id.clone(), new_handle.clone());

		{
			let mut stats = self.stats.write().await;

			stats.total_connections += 1;

			stats.healthy_connections += 1;
		}

		Ok(new_handle)
	}

	async fn start_health_monitoring(&self) {
		let pool = Arc::new(self.clone());

		tokio::spawn(async move {
			let mut interval = interval(Duration::from_millis(pool.config.health_check_interval_ms));

			while *pool.is_running.lock().await {
				interval.tick().await;

				if let Err(e) = pool.check_connection_health().await {
					dev_log!("ipc", "error: [ConnectionPool] Health check failed: {}", e);
				}
			}
		});
	}

	async fn start_connection_cleanup(&self) {
		let pool = Arc::new(self.clone());

		tokio::spawn(async move {
			let mut interval = interval(Duration::from_secs(60));

			while *pool.is_running.lock().await {
				interval.tick().await;

				let cleaned_count = pool.cleanup_stale_connections().await;
				if cleaned_count > 0 {
					dev_log!("ipc", "[ConnectionPool] Cleaned {} stale connections", cleaned_count);
				}
			}
		});
	}

	async fn initialize_min_connections(&self) {
		let current_count = self.connections.lock().await.len();

		if current_count < self.config.min_connections {
			let needed = self.config.min_connections - current_count;

			for _ in 0..needed {
				let handle = ConnectionHandle::new();

				let mut connections = self.connections.lock().await;

				connections.insert(handle.id.clone(), handle);
			}

			dev_log!("ipc", "[ConnectionPool] Initialized {} minimum connections", needed);
		}
	}

	async fn check_connection_health(&self) -> Result<(), String> {
		let mut connections = self.connections.lock().await;

		let mut _health_checker = self.health_checker.lock().await;

		let mut healthy_count = 0;

		for (_id, handle) in connections.iter_mut() {
			let is_healthy = _health_checker.check_connection_health(handle).await;

			handle.update_health(is_healthy);

			if handle.is_healthy() {
				healthy_count += 1;
			}
		}

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

	pub async fn cleanup_stale_connections(&self) -> usize {
		let mut connections = self.connections.lock().await;

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

		{
			let mut stats = self.stats.write().await;

			stats.total_connections = connections.len();

			stats.healthy_connections = connections.values().filter(|h| h.is_healthy()).count();
		}

		stale_ids.len()
	}

	pub async fn get_stats(&self) -> PoolStats { self.stats.read().await.clone() }

	pub async fn get_active_count(&self) -> usize { self.stats.read().await.active_connections }

	pub async fn get_healthy_count(&self) -> usize { self.stats.read().await.healthy_connections }

	pub async fn is_running(&self) -> bool { *self.is_running.lock().await }

	pub fn default_pool() -> Self { Self::new(PoolConfig::default()) }

	pub fn high_performance_pool() -> Self {
		Self::new(PoolConfig {
			max_connections:50,
			min_connections:10,
			connection_timeout_ms:10000,
			max_lifetime_ms:180000,
			idle_timeout_ms:30000,
			health_check_interval_ms:15000,
		})
	}

	pub fn conservative_pool() -> Self {
		Self::new(PoolConfig {
			max_connections:5,
			min_connections:1,
			connection_timeout_ms:60000,
			max_lifetime_ms:600000,
			idle_timeout_ms:120000,
			health_check_interval_ms:60000,
		})
	}

	pub fn calculate_optimal_pool_size() -> usize {
		let num_cpus = num_cpus::get();

		(num_cpus * 2).max(4).min(50)
	}
}

impl Clone for Struct {
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
