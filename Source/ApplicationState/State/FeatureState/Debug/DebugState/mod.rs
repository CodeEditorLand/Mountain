pub mod RegisterDebugConfigurationProvider;
pub mod GetDebugConfigurationProvider;
pub mod RegisterDebugAdapterDescriptorFactory;
pub mod GetDebugAdapterDescriptorFactory;
pub mod GetAllDebugConfigurationProviders;
pub mod GetAllDebugAdapterDescriptorFactories;
pub mod RegisterDebugSession;
pub mod GetDebugSession;
pub mod UnregisterDebugSession;
pub mod GetAllDebugSessions;

use std::{
	collections::HashMap,
	sync::{Arc, Mutex as StandardMutex},
};
use crate::dev_log;

/// Debug configuration provider registration info
#[derive(Clone, Debug)]
pub struct DebugConfigurationProviderRegistration {
	/// The provider handle
	pub ProviderHandle:u32,

	/// The sidecar identifier hosting this provider
	pub SideCarIdentifier:String,
}

/// Debug adapter descriptor factory registration info
#[derive(Clone, Debug)]
pub struct DebugAdapterDescriptorFactoryRegistration {
	/// The factory handle
	pub FactoryHandle:u32,

	/// The sidecar identifier hosting this factory
	pub SideCarIdentifier:String,
}

/// Active debug session entry. Lives in the `DebugSessions` map keyed by
/// session-id (`Uuid::new_v4()` allocated by `DebugProvider::StartDebugging`)
/// so subsequent `SendCommand` calls can resolve the writer end of the
/// spawned adapter's stdin pipe and `DisposeSession` can kill the process.
/// `StdinSender` is `None` for debug-types whose adapter descriptor wasn't
/// of the executable kind we know how to spawn (TCP `server` descriptors,
/// `inlineImplementation` descriptors handled entirely in Cocoon, etc.).
/// In those cases we still record the session so command routing can fall
/// through to a reverse-RPC into Cocoon instead of dropping silently.
#[derive(Clone)]
pub struct DebugSessionEntry {
	/// Session ID assigned at `StartDebugging` time.
	pub SessionId:String,

	/// Debug type (e.g. `"node"`, `"chrome"`) - mirrors the configuration
	/// `type` field, used for diagnostics and routing.
	pub DebugType:String,

	/// Sidecar that owns the configuration-provider / adapter-descriptor
	/// factory. Used for reverse-RPC dispatch when the adapter is not a
	/// spawned executable.
	pub SideCarIdentifier:String,

	/// Channel that writes raw DAP frame bytes to the adapter's stdin.
	/// `None` for non-executable adapter kinds.
	pub StdinSender:Option<tokio::sync::mpsc::UnboundedSender<Vec<u8>>>,

	/// PID of the spawned adapter process (when applicable). `None` for
	/// non-executable kinds. Mountain doesn't keep a live `Child` handle
	/// here because `Child` isn't `Clone`; the process termination is
	/// signalled via dropping `StdinSender`, which the spawn's
	/// stdout/stderr drain tasks treat as shutdown.
	pub ChildPid:Option<u32>,
}

/// Debug state containing debug provider registrations.
#[derive(Clone)]
pub struct Struct {
	/// Debug configuration providers organized by debug type.
	pub DebugConfigurationProviders:Arc<StandardMutex<HashMap<String, DebugConfigurationProviderRegistration>>>,

	/// Debug adapter descriptor factories organized by debug type.
	pub DebugAdapterDescriptorFactories:Arc<StandardMutex<HashMap<String, DebugAdapterDescriptorFactoryRegistration>>>,

	/// Active debug sessions indexed by session-id. Populated by
	/// `DebugProvider::StartDebugging` after the adapter is resolved
	/// (and optionally spawned); removed by `DebugProvider::StopDebugging`
	/// or when the adapter exits. `SendCommand` reads this map to find
	/// the writer for the targeted session.
	pub DebugSessions:Arc<StandardMutex<HashMap<String, DebugSessionEntry>>>,
}



