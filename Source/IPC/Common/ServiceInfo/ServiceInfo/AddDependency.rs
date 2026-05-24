//! `ServiceInfo::AddDependency`

use super::Struct;
use std::time::{Duration, Instant};
use serde::Serialize;
use crate::IPC::Common::ServiceInfo::{ServiceEndpoint, ServicePerformance, ServiceState};

pub fn Fn(This:&mut Struct, Dependency:impl Into<String>) { This.Dependencies.push(Dependency.into()); }
