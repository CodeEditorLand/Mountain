//! Mountain's implementation of [`::Vine::Host::VineHost`] for
//! [`MountainVinegRPCService`]. Lets the canonical handler tree in the
//! Vine crate operate against `&dyn VineHost` while reusing Mountain's
//! `AppHandle`-based `emit` plumbing and IPC bus.

use std::sync::Arc;

use serde_json::Value;
use tauri::{AppHandle, Emitter};

use ::Vine::Host::{ApplicationStateAccess, IPCProvider, RendererEmitter, VineHost};

use crate::{Vine::Server::MountainVinegRPCService::MountainVinegRPCService, dev_log};

/// Minimal `ApplicationStateAccess` carrier for the Mountain embedder.
/// Vine handlers only need the embedder label today; richer state lives
/// behind Mountain-local sub-traits added as port families need them.
struct MountainApplicationStateAccess;

impl ApplicationStateAccess for MountainApplicationStateAccess {
	fn EmbedderName(&self) -> &'static str { "Mountain" }
}

static MOUNTAIN_APP_STATE: MountainApplicationStateAccess = MountainApplicationStateAccess;

/// Cheap-to-clone renderer event sink. Internally just holds a
/// [`tauri::AppHandle`], which is itself a thin `Arc` wrapper - cloning
/// is a ref-count bump. Used by Vine handlers with long-lived flushers
/// (`ProgressReport`, `DecorationTypeLifecycle`,
/// `OutputChannelCoalesce`) that need to emit from a background task.
pub struct TauriRendererEmitter {
	Handle:AppHandle,
}

impl TauriRendererEmitter {
	pub fn New(Handle:AppHandle) -> Self { Self { Handle } }
}

impl RendererEmitter for TauriRendererEmitter {
	fn Emit(&self, Channel:&str, Payload:Value) {
		if let Err(Error) = self.Handle.emit(Channel, Payload) {
			dev_log!("sky-emit", "[SkyEmit] fail channel={} error={}", Channel, Error);
		}
	}
}

/// No-op IPC bridge. Vine handlers reach the real Mountain IPC bus
/// through a dedicated bridge in a follow-up port slice; today's
/// notification handlers do not re-enter the bus, so an inert provider
/// is enough to satisfy the trait surface.
struct InertIPCProvider;

impl IPCProvider for InertIPCProvider {
	fn SendRequest(&self, Channel:&str, _Payload:Value) -> futures::future::BoxFuture<'_, ::Vine::Error::Result<Value>> {
		let Channel = Channel.to_string();

		Box::pin(async move {
			dev_log!("grpc", "warn: [VineHost] InertIPCProvider::SendRequest channel={} swallowed", Channel);

			Ok(Value::Null)
		})
	}

	fn SendNotification(&self, Channel:&str, _Payload:Value) {
		dev_log!("grpc", "warn: [VineHost] InertIPCProvider::SendNotification channel={} swallowed", Channel);
	}
}

impl VineHost for MountainVinegRPCService {
	fn ApplicationState(&self) -> &dyn ApplicationStateAccess { &MOUNTAIN_APP_STATE }

	fn EmitToRenderer(&self, Channel:&str, Payload:Value) {
		if let Err(Error) = self.ApplicationHandle().emit(Channel, Payload) {
			dev_log!("sky-emit", "[SkyEmit] fail channel={} error={}", Channel, Error);
		}
	}

	fn RendererEmitter(&self) -> Arc<dyn RendererEmitter> {
		Arc::new(TauriRendererEmitter::New(self.ApplicationHandle().clone()))
	}

	fn IPCProvider(&self) -> Arc<dyn IPCProvider> { Arc::new(InertIPCProvider) }
}
