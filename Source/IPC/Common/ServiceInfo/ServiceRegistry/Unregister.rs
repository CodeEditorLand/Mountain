//! `ServiceRegistry::Unregister`

use super::Struct;
use std::{
	collections::HashMap,
	time::{Duration, Instant},
};
use crate::IPC::Common::ServiceInfo::ServiceInfo;

pub fn Fn(This:&mut Struct, Name:&str) -> Option<ServiceInfo::Struct> {
		This.Services.remove(Name).map(|Service| {
			This.LastDiscovery = Instant::now();
			Service
		})
	}
