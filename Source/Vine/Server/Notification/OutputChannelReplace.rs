//! Cocoon → Mountain `outputChannel.replace` notification.
//!
//! Atomic buffer replacement: equivalent to a `clear` followed by an
//! `append` of `value`, except the workbench renders both as a single
//! frame so the user doesn't see a momentary flash of empty content.
//! Forwarded to Sky as `sky://output/replace` where the workbench's
//! channel buffer is rebuilt in a single tick.

use serde_json::Value;

use super::Support::RelayToSky::RelayToSky;
use crate::Vine::Server::MountainVinegRPCService::MountainVinegRPCService;

pub async fn OutputChannelReplace(Service:&MountainVinegRPCService, Parameter:&Value) {
	RelayToSky(Service, "sky://output/replace", Parameter, "grpc", "[OutputChannel] replace");
}
