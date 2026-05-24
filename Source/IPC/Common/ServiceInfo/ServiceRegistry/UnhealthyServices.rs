//! `ServiceRegistry::UnhealthyServices`

use super::Struct;
use std::{
	collections::HashMap,
	time::{Duration, Instant},
};
use crate::IPC::Common::ServiceInfo::ServiceInfo;

pub fn Fn(This:&Struct) -> Vec<&ServiceInfo::Struct> {
		This.Services.values().filter(|S| !S.IsHealthy()).collect()
	}
