#![allow(non_snake_case)]

//! Per-connection health-probe helper used by
//! `Pool::Struct::start_health_monitoring`. Currently runs a
//! simulated 10ms ping; real implementations would send a
//! protocol-level keepalive.

use std::time::{Duration, Instant};

use crate::IPC::Enhanced::ConnectionPool::ConnectionHandle;

pub struct Struct {
	pub(super) ping_timeout:Duration,
}

impl Struct {
	pub(super) fn new() -> Self { Self { ping_timeout:Duration::from_secs(5) } }

	pub(super) async fn check_connection_health(&self, _handle:&mut ConnectionHandle::Struct) -> bool {
		let start_time = Instant::now();
		tokio::time::sleep(Duration::from_millis(10)).await;
		let response_time = start_time.elapsed();
		response_time < self.ping_timeout
	}
}
