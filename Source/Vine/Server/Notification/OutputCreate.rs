use serde_json::Value;

use super::Support::Fn::Fn;
use crate::Vine::Server::MountainVinegRPCService::Struct;

pub async fn Fn(Service:&MountainVinegRPCService, Parameter:&Value) {
	RelayToSky(Service, "sky://output/create", Parameter, "grpc", "[Output] create");
}
