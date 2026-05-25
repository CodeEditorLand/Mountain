//! Cocoon → Mountain `tree.refresh` notification.
//!
//! Fired when an extension's `TreeDataProvider.onDidChangeTreeData` event
//! emitter fires. The payload `{ handle, viewId, element }` is relayed to
//! Sky as `sky://tree-view/refresh`; Sky's handler resolves the view via
//! `TreeViewByViewId(viewId)` and calls `ITreeView.refresh()` which
//! triggers a fresh `getChildren()` round-trip back to the extension.

use serde_json::Value;

use super::Support::RelayToSky::RelayToSky;
use crate::Vine::Server::MountainVinegRPCService::MountainVinegRPCService;

pub async fn TreeRefresh(Service:&MountainVinegRPCService, Parameter:&Value) {
	RelayToSky(Service, "sky://tree-view/refresh", Parameter, "grpc", "[Tree] refresh");
}
