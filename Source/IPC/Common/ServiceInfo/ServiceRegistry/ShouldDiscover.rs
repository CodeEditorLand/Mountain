//! `ServiceRegistry::ShouldDiscover`

use super::Struct;
use std::{
	collections::HashMap,
	time::{Duration, Instant},
};
use crate::IPC::Common::ServiceInfo::ServiceInfo;

pub fn Fn(This:&Struct) -> bool { This.LastDiscovery.elapsed() >= This.DiscoveryInterval }
