pub mod New;
pub mod UpdateState;
pub mod RecordHeartbeat;
pub mod IsHealthy;
pub mod AddDependency;

use std::time::{Duration, Instant};
use serde::Serialize;
use crate::IPC::Common::ServiceInfo::{ServiceEndpoint, ServicePerformance, ServiceState};

#[derive(Debug, Clone, Serialize)]
pub struct Struct {
	pub Name:String,

	pub Version:String,

	pub State:ServiceState::Enum,

	#[serde(skip)]
	pub StateSince:Instant,

	pub Uptime:Duration,

	#[serde(skip)]
	pub LastHeartbeat:Option<Instant>,

	pub Dependencies:Vec<String>,

	pub Performance:ServicePerformance::Struct,

	pub Endpoint:Option<ServiceEndpoint::Struct>,
}
