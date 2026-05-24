//! `ServiceRegistry::Register`

use super::Struct;
use std::{
	collections::HashMap,
	time::{Duration, Instant},
};
use crate::IPC::Common::ServiceInfo::ServiceInfo;

pub fn Fn(This:&mut Struct, Service:ServiceInfo::Struct) {
		This.Services.insert(Service.Name.clone(), Service);

		This.LastDiscovery = Instant::now();
	}
