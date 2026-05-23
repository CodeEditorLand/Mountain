//! Per-service descriptor: name, version, lifecycle state, performance
//! counters, dependency list, optional endpoint. Health is the
//! conjunction of operational state, recent heartbeat (≤ 30s), and
//! error rate ≤ 10 %.

use std::time::{Duration, Instant};

use serde::Serialize;

use crate::IPC::Common::ServiceInfo::{ServiceEndpoint, ServicePerformance, ServiceState};

#[derive(Debug, Clone, Serialize)]
pub struct Struct {
	pub Name:String,

	pub Version:String,

	pub State:ServiceState::Enum,

	#[serde(skip)]
	pub StateSince:Instant,

	pub Uptime:Duration,

	#[serde(skip)]
	pub LastHeartbeat:Option<Instant>,

	pub Dependencies:Vec<String>,

	pub Performance:ServicePerformance::Struct,

	pub Endpoint:Option<ServiceEndpoint::Struct>,
}

impl Struct {
	pub fn new(Name:impl Into<String>, Version:impl Into<String>) -> Self {
		Self {
			Name:Name.into(),

			Version:Version.into(),

			State:ServiceState::Enum::Starting,

			StateSince:Instant::now(),

			Uptime:Duration::ZERO,

			LastHeartbeat:None,

			Dependencies:Vec::new(),

			Performance:ServicePerformance::Struct::new(),

			Endpoint:None,
		}
	}

	pub fn UpdateState(&mut self, NewState:ServiceState::Enum) {
		self.State = NewState;

		self.StateSince = Instant::now();
	}

	pub fn RecordHeartbeat(&mut self) {
		self.LastHeartbeat = Some(Instant::now());

		if self.State == ServiceState::Enum::Running {
			self.Uptime = self.StateSince.elapsed();
		}
	}

	pub fn IsHealthy(&self) -> bool {
		if !self.State.IsOperational() {
			return false;
		}

		if let Some(Heartbeat) = self.LastHeartbeat
			&& Heartbeat.elapsed() > Duration::from_secs(30)
		{
			return false;
		}

		if self.Performance.ErrorRate() > 0.1 {
			return false;
		}

		true
	}

	pub fn AddDependency(&mut self, Dependency:impl Into<String>) { self.Dependencies.push(Dependency.into()); }
}
