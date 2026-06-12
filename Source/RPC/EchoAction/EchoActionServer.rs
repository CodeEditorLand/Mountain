//! Singleton submission gate for every extension-host → Mountain request. Wraps
//! the Echo scheduler with a per-method priority lane.
use std::sync::Arc;

use Echo::{Scheduler::Scheduler::Scheduler, Task::Priority::Priority as EchoPriority};
use tokio::sync::oneshot;

use crate::RPC::EchoAction::{ExtensionHostRegistry, ResolveMethodPriority};

/// Echo action server: singleton submission gate that schedules extension host
/// requests on an Echo work-stealing scheduler.
#[derive(Clone)]
pub struct Struct {
	Registry:Arc<ExtensionHostRegistry::Struct>,
}

impl Default for Struct {
	fn default() -> Self { Self::new() }
}

/// Creates a new `EchoActionServer` with an empty registry.
impl Struct {
	pub fn new() -> Self { Self { Registry:Arc::new(ExtensionHostRegistry::Struct::new()) } }

	/// Registry accessor so tonic handlers can pass it into per-extension
	/// logic without threading it through the scheduler.
	pub fn Registry(&self) -> Arc<ExtensionHostRegistry::Struct> { self.Registry.clone() }

	/// Dispatch `Task` to the Echo scheduler on the lane chosen for `Method`,
	/// wait for completion, and return the result.
	pub async fn Dispatch<F, T>(&self, Scheduler:&Scheduler, Method:&str, Task:F) -> Result<T, String>
	where
		F: std::future::Future<Output = T> + Send + 'static,
		T: Send + 'static, {
		let Priority = ResolveMethodPriority::Fn(Method);

		let (Sender, Receiver) = oneshot::channel::<T>();

		Scheduler.Submit(
			async move {
				let Output = Task.await;

				let _ = Sender.send(Output);
			},
			Priority,
		);

		Receiver
			.await
			.map_err(|_| "EchoAction task cancelled before completion".to_string())
	}
}

// Allow `EchoPriority` import below to satisfy clippy unused warning when
// the inner Scheduler import is feature-gated in future revisions.
fn _Priority(P:EchoPriority) -> EchoPriority { P }
