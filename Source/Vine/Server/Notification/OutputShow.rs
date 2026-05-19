#![allow(non_snake_case)]
use serde_json::Value;

use super::Support::RelayToSky::RelayToSky;
use crate::Vine::Server::MountainVinegRPCService::MountainVinegRPCService;
pub async fn OutputShow(Service:&MountainVinegRPCService, Parameter:&Value) {
	RelayToSky(Service, "sky://output/show", Parameter, "grpc", "[Output] show");
}
