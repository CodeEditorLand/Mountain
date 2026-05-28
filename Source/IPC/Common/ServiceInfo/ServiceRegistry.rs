//! Map of registered services keyed by name + a configurable discovery
//! cadence. `ShouldDiscover` returns true once the configured interval
//! has elapsed since the last `MarkDiscovery` (or `Register` /
//! `Unregister`, both of which stamp the timestamp implicitly).

use std::{
	collections::HashMap,
	time::{Duration, Instant},
};

use crate::IPC::Common::ServiceInfo::ServiceInfo;

#[derive(Debug, Clone)]
pub struct Struct {
	pub Services:HashMap<String, ServiceInfo::Struct>,

	pub LastDiscovery:Instant,

	pub DiscoveryInterval:Duration,
}

impl Struct {
	pub fn new(DiscoveryInterval:Duration) -> Self {
		Self { Services:HashMap::new(), LastDiscovery:Instant::now(), DiscoveryInterval }
	}

	pub fn Register(&mut self, Service:ServiceInfo::Struct) {
		self.Services.insert(Service.Name.clone(), Service);

		self.LastDiscovery = Instant::now();
	}

	pub fn Unregister(&mut self, Name:&str) -> Option<ServiceInfo::Struct> {
		self.Services.remove(Name).map(|Service| {
			self.LastDiscovery = Instant::now();

			Service
		})
	}

	pub fn Get(&self, Name:&str) -> Option<&ServiceInfo::Struct> { self.Services.get(Name) }

	pub fn GetMut(&mut self, Name:&str) -> Option<&mut ServiceInfo::Struct> { self.Services.get_mut(Name) }

	pub fn ShouldDiscover(&self) -> bool { self.LastDiscovery.elapsed() >= self.DiscoveryInterval }

	pub fn HealthyServices(&self) -> Vec<&ServiceInfo::Struct> {
		self.Services.values().filter(|S| S.IsHealthy()).collect()
	}

	pub fn UnhealthyServices(&self) -> Vec<&ServiceInfo::Struct> {
		self.Services.values().filter(|S| !S.IsHealthy()).collect()
	}

	pub fn MarkDiscovery(&mut self) { self.LastDiscovery = Instant::now(); }
}

impl Default for Struct {
	fn default() -> Self { Self::new(Duration::from_secs(60)) }
}
