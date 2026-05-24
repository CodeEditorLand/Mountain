//! `ServiceRegistry::MarkDiscovery`

use super::Struct;
use std::{
	collections::HashMap,
	time::{Duration, Instant},
};
use crate::IPC::Common::ServiceInfo::ServiceInfo;

pub fn Fn(This:&mut Struct) { This.LastDiscovery = Instant::now(); }
