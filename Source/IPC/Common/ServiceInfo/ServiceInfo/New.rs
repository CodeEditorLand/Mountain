//! `ServiceInfo::New`

use super::Struct;
use std::time::{Duration, Instant};
use serde::Serialize;
use crate::IPC::Common::ServiceInfo::{ServiceEndpoint, ServicePerformance, ServiceState};

pub fn Fn(Name:impl Into<String>, Version:impl Into<String>) -> Struct {
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
