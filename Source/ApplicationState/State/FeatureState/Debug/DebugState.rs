//! # DebugState Module (ApplicationState)
//!
//! ## RESPONSIBILITIES
//! Manages debug provider state including debug configuration providers and
//! adapter descriptor factories.
//!
//! ## ARCHITECTURAL ROLE
//! DebugState is part of the **FeatureState** module, storing debug provider
//! registrations keyed by debug type.
//!
//! ## KEY COMPONENTS
//! - DebugState: Main struct containing debug provider registrations
//! - Default: Initialization implementation
//! - Helper methods: Debug registration management
//!
//! ## ERROR HANDLING
//! - Thread-safe access via `Arc<Mutex<...>>`
//!
//! ## LOGGING
//! State changes are logged at appropriate levels (debug, info, warn, error).
//!
//! ## PERFORMANCE CONSIDERATIONS
//! - Lock mutexes briefly and release immediately
//! - Use Arc for shared ownership across threads

use std::{collections::HashMap, sync::Arc};

use parking_lot::Mutex;

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
///
/// `StdinSender` is `None` for debug-types whose adapter descriptor wasn't
/// of the executable kind we know how to spawn (TCP `server` descriptors,
/// `inlineImplementation` descriptors handled entirely in Cocoon, etc.).
/// In those cases we still record the session so command routing can fall
/// through to a reverse-RPC into Cocoon instead of dropping silently.
#[derive(Clone)]
pub struct DebugSessionEntry {
	/// Session ID assigned at `StartDebugging` time.
	pub SessionId:String,

	/// Debug type (e.g. `node`, `chrome`) - mirrors the configuration
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

/// A single breakpoint stored in Mountain's debug state. Mirrors the
/// shape VS Code's `debug:addBreakpoints` IPC argument carries.
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct BreakpointEntry {
	/// Opaque identifier assigned by VS Code (a UUID string).
	pub id:String,

	/// Breakpoint kind: `"source"`, `"function"`, `"data"`, etc.
	#[serde(default)]
	pub kind:String,

	/// Source URI string (present for source breakpoints).
	#[serde(default)]
	pub uri:String,

	/// 1-based line number (present for source breakpoints).
	#[serde(default)]
	pub line:u64,

	/// 1-based column (optional, source breakpoints).
	#[serde(default)]
	pub column:Option<u64>,

	/// Whether the breakpoint is currently enabled.
	#[serde(default = "default_true")]
	pub enabled:bool,

	/// Raw JSON value of the full breakpoint payload, preserved so
	/// `debug:getBreakpoints` can round-trip the original shape.
	#[serde(skip)]
	pub raw:serde_json::Value,
}

fn default_true() -> bool { true }

/// Debug state containing debug provider registrations.
#[derive(Clone)]
pub struct DebugState {
	/// Debug configuration providers organized by debug type.
	pub DebugConfigurationProviders:Arc<Mutex<HashMap<String, DebugConfigurationProviderRegistration>>>,

	/// Debug adapter descriptor factories organized by debug type.
	pub DebugAdapterDescriptorFactories:Arc<Mutex<HashMap<String, DebugAdapterDescriptorFactoryRegistration>>>,

	/// Active debug sessions indexed by session-id. Populated by
	/// `DebugProvider::StartDebugging` after the adapter is resolved
	/// (and optionally spawned); removed by `DebugProvider::StopDebugging`
	/// or when the adapter exits. `SendCommand` reads this map to find
	/// the writer for the targeted session.
	pub DebugSessions:Arc<Mutex<HashMap<String, DebugSessionEntry>>>,

	/// Breakpoints registered via `debug:addBreakpoints`, keyed by
	/// breakpoint id. Mountain keeps this as the authoritative store so
	/// `debug:getBreakpoints` can be served locally without a Cocoon
	/// round-trip.
	pub Breakpoints:Arc<Mutex<HashMap<String, BreakpointEntry>>>,
}

impl Default for DebugState {
	fn default() -> Self {
		dev_log!("exthost", "[DebugState] Initializing default debug state...");

		Self {
			DebugConfigurationProviders:Arc::new(Mutex::new(HashMap::new())),

			DebugAdapterDescriptorFactories:Arc::new(Mutex::new(HashMap::new())),

			DebugSessions:Arc::new(Mutex::new(HashMap::new())),

			Breakpoints:Arc::new(Mutex::new(HashMap::new())),
		}
	}
}

impl DebugState {
	/// Registers a debug configuration provider.
	pub fn RegisterDebugConfigurationProvider(
		&self,

		debug_type:String,

		provider_handle:u32,

		sidecar_identifier:String,
	) -> Result<(), String> {
		let mut guard = self.DebugConfigurationProviders.lock();

		guard.insert(
			debug_type,
			DebugConfigurationProviderRegistration {
				ProviderHandle:provider_handle,
				SideCarIdentifier:sidecar_identifier,
			},
		);

		Ok(())
	}

	/// Gets a debug configuration provider registration by debug type.
	pub fn GetDebugConfigurationProvider(&self, debug_type:&str) -> Option<DebugConfigurationProviderRegistration> {
		self.DebugConfigurationProviders.lock().get(debug_type).cloned()
	}

	/// Registers a debug adapter descriptor factory.
	pub fn RegisterDebugAdapterDescriptorFactory(
		&self,

		debug_type:String,

		factory_handle:u32,

		sidecar_identifier:String,
	) -> Result<(), String> {
		let mut guard = self.DebugAdapterDescriptorFactories.lock();

		guard.insert(
			debug_type,
			DebugAdapterDescriptorFactoryRegistration {
				FactoryHandle:factory_handle,
				SideCarIdentifier:sidecar_identifier,
			},
		);

		Ok(())
	}

	/// Gets a debug adapter descriptor factory registration by debug type.
	pub fn GetDebugAdapterDescriptorFactory(
		&self,

		debug_type:&str,
	) -> Option<DebugAdapterDescriptorFactoryRegistration> {
		self.DebugAdapterDescriptorFactories.lock().get(debug_type).cloned()
	}

	/// Gets all registered debug configuration providers.
	pub fn GetAllDebugConfigurationProviders(&self) -> HashMap<String, DebugConfigurationProviderRegistration> {
		self.DebugConfigurationProviders.lock().clone()
	}

	/// Gets all registered debug adapter descriptor factories.
	pub fn GetAllDebugAdapterDescriptorFactories(&self) -> HashMap<String, DebugAdapterDescriptorFactoryRegistration> {
		self.DebugAdapterDescriptorFactories.lock().clone()
	}

	/// Records an active debug session. Replaces any prior entry under the
	/// same `SessionId` (defensive: shouldn't happen since IDs are uuids).
	pub fn RegisterDebugSession(&self, Entry:DebugSessionEntry) -> Result<(), String> {
		let mut Guard = self.DebugSessions.lock();

		Guard.insert(Entry.SessionId.clone(), Entry);

		Ok(())
	}

	/// Resolves an active session by id. Returns a `Clone` so the caller
	/// can drop the lock before doing IO with the entry's `StdinSender`.
	pub fn GetDebugSession(&self, SessionId:&str) -> Option<DebugSessionEntry> {
		self.DebugSessions.lock().get(SessionId).cloned()
	}

	/// Removes a session from the registry. Dropping the returned entry's
	/// `StdinSender` triggers the adapter-spawn drain tasks to wind down
	/// (their `recv()` returns `None`) which closes the adapter stdin and
	/// the adapter shuts itself down.
	pub fn UnregisterDebugSession(&self, SessionId:&str) -> Option<DebugSessionEntry> {
		let mut Guard = self.DebugSessions.lock();

		Guard.remove(SessionId)
	}

	/// Snapshot of all active sessions. Used by diagnostic dev_log surfaces
	/// and the reverse-RPC dispatch when no session-id is supplied.
	pub fn GetAllDebugSessions(&self) -> HashMap<String, DebugSessionEntry> { self.DebugSessions.lock().clone() }

	/// Inserts or replaces breakpoints from a `debug:addBreakpoints` call.
	/// Each entry's `raw` field is set to the original JSON value so
	/// `GetBreakpoints` can round-trip the full payload shape VS Code expects.
	pub fn AddBreakpoints(&self, Entries:Vec<BreakpointEntry>) {
		let mut Guard = self.Breakpoints.lock();

		for Entry in Entries {
			Guard.insert(Entry.id.clone(), Entry);
		}
	}

	/// Removes breakpoints by id. Ids not present are silently skipped.
	pub fn RemoveBreakpoints(&self, Ids:&[String]) {
		let mut Guard = self.Breakpoints.lock();

		for Id in Ids {
			Guard.remove(Id.as_str());
		}
	}

	/// Returns the raw JSON values for all stored breakpoints, suitable for
	/// returning directly from `debug:getBreakpoints`.
	pub fn GetBreakpoints(&self) -> Vec<serde_json::Value> {
		self.Breakpoints.lock().values().map(|E| E.raw.clone()).collect()
	}
}
