use serde_json::Value;

use crate::Vine::Server::MountainVinegRPCService::MountainVinegRPCService;

pub fn RelayToSky(Service:&MountainVinegRPCService, SkyEvent:&str, Parameter:&Value, LogTag:&str, LogLine:&str) {

	::Vine::Server::Notification::Support::RelayToSky::Fn(Service, SkyEvent, Parameter, LogTag, LogLine);
}
