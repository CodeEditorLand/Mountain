use serde_json::Value;

use super::Support::RelayToSky::RelayToSky;
use crate::Vine::Server::MountainVinegRPCService::MountainVinegRPCService;

pub async fn OutputAppend(Service:&MountainVinegRPCService, Parameter:&Value) {
	RelayToSky(Service, "sky://output/append", Parameter, "grpc", "[Output] append");
}
