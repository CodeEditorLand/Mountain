#![allow(non_snake_case)]

//! Per-connection state - id, lifecycle timestamps, rolling
//! health score, error / success counters, and a
//! `ConnectionHealth::Enum` summary. `update_health` adjusts
//! the score on each operation; `is_healthy` decides whether
//! the pool can hand the connection out.

use std::time::{Duration, Instant};

use uuid::Uuid;

use crate::IPC::Enhanced::ConnectionPool::ConnectionHealth;

#[derive(Debug, Clone)]
pub struct Struct {

	pub id:String,

	pub created_at:Instant,

	pub last_used:Instant,

	pub health_score:f64,

	pub error_count:usize,

	pub successful_operations:usize,

	pub total_operations:usize,

	pub is_active:bool,

	pub reuse_count:u32,

	pub health:ConnectionHealth::Enum,
}

impl Struct {

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

			reuse_count:0,

			health:ConnectionHealth::Enum::Healthy,
		}
	}

	pub fn update_health(&mut self, success:bool) {

		self.last_used = Instant::now();

		self.total_operations += 1;

		if success {

			self.successful_operations += 1;

			self.health_score = (self.health_score + 2.0).min(100.0);

			self.error_count = 0;
		} else {

			self.error_count += 1;

			self.health_score = (self.health_score - 10.0).max(0.0);
		}

		let success_rate = if self.total_operations > 0 {

			self.successful_operations as f64 / self.total_operations as f64
		} else {

			1.0
		};

		self.health_score = (self.health_score * 0.7 + success_rate * 100.0 * 0.3).max(0.0).min(100.0);
	}

	pub fn is_healthy(&self) -> bool {

		self.health_score > 50.0 && self.error_count < 5 && self.is_active && self.age().as_secs() < 300
	}

	pub fn age(&self) -> Duration { self.created_at.elapsed() }

	pub fn idle_time(&self) -> Duration { self.last_used.elapsed() }

	pub fn success_rate(&self) -> f64 {

		if self.total_operations == 0 {

			1.0
		} else {

			self.successful_operations as f64 / self.total_operations as f64
		}
	}
}
