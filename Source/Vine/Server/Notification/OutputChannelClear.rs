use serde_json::Value;

use super::Support::RelayToSky::RelayToSky;
use crate::Vine::Server::MountainVinegRPCService::MountainVinegRPCService;

pub async fn OutputChannelClear(Service:&MountainVinegRPCService, Parameter:&Value) {
	RelayToSky(Service, "sky://output/clear", Parameter, "grpc", "[OutputChannel] clear");
}
