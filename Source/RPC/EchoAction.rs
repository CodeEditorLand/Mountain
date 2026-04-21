//! # EchoAction — Cocoon → Mountain routing through Echo (Atom O4)
//!
//! Every request that Cocoon sends back into Mountain (via `MountainService`
//! gRPC) used to execute inline on the tonic handler future. That's fine in
//! the steady state but pathological under bursts — a `$activateByEvent("*")`
//! fan-out can fire 28+ `ReadFile` + 28+ `Stat` + 28+ `Configuration.Inspect`
//! calls within a few hundred milliseconds and starve any interactive Wind
//! IPC that arrives during the burst.
//!
//! `EchoAction` is the submission point for those inbound requests. The
//! `MountainService` handlers call `EchoAction::Dispatch(runtime, method,
//! task)` which:
//!
//!   1. Maps `method` → `EchoPriority` via a small table keyed on the
//!      `$prefixed` wire-string Cocoon uses.
//!   2. Submits `task` to `runtime.Scheduler` on the chosen lane via a oneshot
//!      channel.
//!   3. Awaits the receiver and returns the result on the tonic future.
//!
//! The `ExtensionHostRegistry` + `ExtensionRouter` stubs remain (they were
//! here when EchoAction was a placeholder); both are wired into the dispatch
//! path so future work can record per-extension-host metrics without a
//! refactor.
//!
//! ## Priority table
//!
//! | Wire method                             | Lane   | Reason                              |
//! | --------------------------------------- | ------ | ----------------------------------- |
//! | `FileSystem.ReadFile` / `WriteFile`     | High   | extension UI waits on it            |
//! | `ShowInformationMessage` / `ShowError…` | High   | user-visible                        |
//! | `ExecuteContributedCommand`             | High   | user action                         |
//! | `RegisterCommand` + Register* providers | Normal | activation path                     |
//! | `Configuration.Inspect`                 | Normal | common, not critical                |
//! | `FindFiles` / `FindTextInFiles`         | Low    | long-running                        |
//! | `GitExec`                               | Low    | spawns subprocess                   |
//! | everything else                         | Normal | safe default                        |

#![allow(non_snake_case)]

use std::{collections::HashMap, sync::Arc};

use Echo::{Scheduler::Scheduler::Scheduler, Task::Priority::Priority as EchoPriority};
use tokio::sync::{RwLock, oneshot};

/// Singleton submission gate for every Cocoon→Mountain request.
#[derive(Clone)]
pub struct EchoActionServer {
	Registry:Arc<ExtensionHostRegistry>,
}

impl Default for EchoActionServer {
	fn default() -> Self { Self::new() }
}

impl EchoActionServer {
	pub fn new() -> Self { Self { Registry:Arc::new(ExtensionHostRegistry::new()) } }

	/// Registry accessor so tonic handlers can pass it into their
	/// per-extension logic without threading it through the scheduler.
	pub fn Registry(&self) -> Arc<ExtensionHostRegistry> { self.Registry.clone() }

	/// Submit `Task` to the Echo scheduler on the lane chosen for `Method`,
	/// wait for its completion, and return the task's result. Cocoon tonic
	/// handlers call this as the first line of their body so every request
	/// lands in Echo's work-stealing queue.
	pub async fn Dispatch<F, T>(&self, Scheduler:&Scheduler, Method:&str, Task:F) -> Result<T, String>
	where
		F: std::future::Future<Output = T> + Send + 'static,
		T: Send + 'static, {
		let Priority = ResolveMethodPriority(Method);
		let (Sender, Receiver) = oneshot::channel::<T>();

		Scheduler.Submit(
			async move {
				let Output = Task.await;

				if Sender.send(Output).is_err() {
					// Receiver dropped — tonic future was cancelled. Fine;
					// nothing to do.
				}
			},
			Priority,
		);

		Receiver
			.await
			.map_err(|_| "EchoAction task cancelled before completion".to_string())
	}
}

/// Map a Cocoon→Mountain wire method name to an Echo priority lane.
fn ResolveMethodPriority(Method:&str) -> EchoPriority {
	match Method {
		// Direct UI waits
		"FileSystem.ReadFile"
		| "FileSystem.WriteFile"
		| "FileSystem.Stat"
		| "ShowInformationMessage"
		| "ShowWarningMessage"
		| "ShowErrorMessage"
		| "ExecuteContributedCommand"
		| "ShowTextDocument" => EchoPriority::High,

		// Long-running / background
		"FindFiles" | "FindTextInFiles" | "GitExec" | "WatchFile" => EchoPriority::Low,

		// Default
		_ => EchoPriority::Normal,
	}
}

/// Tracks which extension host owns which extension id.
///
/// Populated from `$deltaExtensions` + `InitExtensionHost` payloads; read by
/// `ExtensionRouter` when a request needs to be routed to a specific host.
pub struct ExtensionHostRegistry {
	Hosts:Arc<RwLock<HashMap<String, String>>>,
}

impl ExtensionHostRegistry {
	pub fn new() -> Self { Self { Hosts:Arc::new(RwLock::new(HashMap::new())) } }

	pub async fn Record(&self, ExtensionIdentifier:String, HostIdentifier:String) {
		self.Hosts.write().await.insert(ExtensionIdentifier, HostIdentifier);
	}

	pub async fn Forget(&self, ExtensionIdentifier:&str) { self.Hosts.write().await.remove(ExtensionIdentifier); }

	pub async fn Resolve(&self, ExtensionIdentifier:&str) -> Option<String> {
		self.Hosts.read().await.get(ExtensionIdentifier).cloned()
	}

	pub async fn Count(&self) -> usize { self.Hosts.read().await.len() }
}

impl Default for ExtensionHostRegistry {
	fn default() -> Self { Self::new() }
}

/// Pairs an extension identifier with the host that owns it; used by
/// EchoActionServer to scope priority or telemetry when more than one
/// extension host is active (Grove + Cocoon).
pub struct ExtensionRouter {
	registry:Arc<ExtensionHostRegistry>,
}

impl ExtensionRouter {
	pub fn new(registry:Arc<ExtensionHostRegistry>) -> Self { Self { registry } }

	pub async fn HostFor(&self, ExtensionIdentifier:&str) -> Option<String> {
		self.registry.Resolve(ExtensionIdentifier).await
	}
}

#[cfg(test)]
mod Tests {
	use Echo::Task::Priority::Priority as EchoPriority;

	use super::{EchoActionServer, ExtensionHostRegistry, ResolveMethodPriority};

	#[test]
	fn PriorityTable() {
		assert_eq!(ResolveMethodPriority("FileSystem.ReadFile"), EchoPriority::High);
		assert_eq!(ResolveMethodPriority("ShowErrorMessage"), EchoPriority::High);
		assert_eq!(ResolveMethodPriority("FindFiles"), EchoPriority::Low);
		assert_eq!(ResolveMethodPriority("GitExec"), EchoPriority::Low);
		assert_eq!(ResolveMethodPriority("Configuration.Inspect"), EchoPriority::Normal);
		assert_eq!(ResolveMethodPriority("_unknown_method"), EchoPriority::Normal);
	}

	#[tokio::test]
	async fn RegistryRoundTrip() {
		let Registry = ExtensionHostRegistry::new();

		assert_eq!(Registry.Count().await, 0);
		assert_eq!(Registry.Resolve("vscode.git").await, None);

		Registry.Record("vscode.git".to_string(), "cocoon-main".to_string()).await;

		assert_eq!(Registry.Count().await, 1);
		assert_eq!(Registry.Resolve("vscode.git").await.as_deref(), Some("cocoon-main"));

		Registry.Forget("vscode.git").await;

		assert_eq!(Registry.Count().await, 0);
	}

	#[test]
	fn ServerIsClonableAndDefault() {
		let Server = EchoActionServer::default();
		let _Clone = Server.clone();
	}
}
