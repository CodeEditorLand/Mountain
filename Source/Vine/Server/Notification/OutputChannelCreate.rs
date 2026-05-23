use serde_json::Value;

use super::Support::RelayToSky::RelayToSky;
use crate::Vine::Server::MountainVinegRPCService::MountainVinegRPCService;

pub async fn OutputChannelCreate(Service:&MountainVinegRPCService, Parameter:&Value) {
	RelayToSky(
		Service,
		"sky://output/create",
		Parameter,
		"output-verbose",
		"[OutputChannel] create",
	);
}
