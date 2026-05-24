pub mod New;
pub mod Register;
pub mod Unregister;
pub mod Get;
pub mod GetMut;
pub mod ShouldDiscover;
pub mod HealthyServices;
pub mod UnhealthyServices;
pub mod MarkDiscovery;

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
