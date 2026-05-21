#![allow(non_snake_case)]
use serde_json::Value;

use super::Support::RelayToSky::RelayToSky;
use crate::Vine::Server::MountainVinegRPCService::MountainVinegRPCService;

pub async fn ProgressUpdate(Service:&MountainVinegRPCService, Parameter:&Value) {
	RelayToSky(
		Service,
		"sky://notification/progress-update",
		Parameter,
		"grpc",
		"[Progress] update",
	);
}
