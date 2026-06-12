use serde_json::Value;

use crate::Vine::Server::MountainVinegRPCService::MountainVinegRPCService;

/// Forwards a Cocoon notification event to the Sky IPC bridge.
pub fn RelayToSky(Service:&MountainVinegRPCService, SkyEvent:&str, Parameter:&Value, LogTag:&str, LogLine:&str) {
	::Vine::Server::Notification::Support::RelayToSky::Fn(Service, SkyEvent, Parameter, LogTag, LogLine);
}
