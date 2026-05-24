//! `ServiceRegistry::New`

use super::Struct;
use std::{
	collections::HashMap,
	time::{Duration, Instant},
};
use crate::IPC::Common::ServiceInfo::ServiceInfo;

pub fn Fn(DiscoveryInterval:Duration) -> Struct {
		Self { Services:HashMap::new(), LastDiscovery:Instant::now(), DiscoveryInterval }
	}
