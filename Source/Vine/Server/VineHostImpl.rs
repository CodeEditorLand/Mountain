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

static MOUNTAIN_APP_STATE:MountainApplicationStateAccess = MountainApplicationStateAccess;

/// Cheap-to-clone renderer event sink. Internally holds a
/// [`tauri::AppHandle`], which is itself a thin `Arc` wrapper - cloning
/// is a ref-count bump. Used by Vine handlers with long-lived flushers
/// (`ProgressReport`, `DecorationTypeLifecycle`, `OutputChannelCoalesce`,
/// `SetTextEditorDecorations`, `RegisterCommand`) that emit from a
/// background task.
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

/// IPC bridge that routes `SendNotification` calls to the Vine gRPC client
/// so breakpoint fan-backs and similar cross-extension notifications reach
/// Cocoon. `SendRequest` is left as a no-op until a handler needs it.
struct MountainIPCProvider;

impl IPCProvider for MountainIPCProvider {
	fn SendRequest(
		&self,
		Channel:&str,
		_Payload:Value,
	) -> futures::future::BoxFuture<'_, ::Vine::Error::Result<Value>> {
		let Channel = Channel.to_string();

		Box::pin(async move {
			dev_log!("grpc", "warn: [VineHost] IPCProvider::SendRequest channel={} - not wired", Channel);

			Ok(Value::Null)
		})
	}

	fn SendNotification(&self, Channel:&str, Method:&str, Payload:Value) {
		let Ch = Channel.to_string();
		let M = Method.to_string();

		tauri::async_runtime::spawn(async move {
			let _ = crate::Vine::Client::SendNotification::Fn(Ch, M, Payload).await;
		});
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

	fn IPCProvider(&self) -> Arc<dyn IPCProvider> { Arc::new(MountainIPCProvider) }

	fn UnregisterProvider(&self, Handle:u32) {
		self.RunTime()
			.Environment
			.ApplicationState
			.Extension
			.ProviderRegistration
			.UnregisterProvider(Handle);
	}

	fn RegisterCommandInRegistry(&self, CommandId:&str, SideCarIdentifier:&str) {
		use crate::Environment::CommandProvider::CommandHandler;

		if let Ok(mut Registry) = self
			.RunTime()
			.Environment
			.ApplicationState
			.Extension
			.Registry
			.CommandRegistry
			.lock()
		{
			Registry.insert(
				CommandId.to_string(),
				CommandHandler::Proxied {
					SideCarIdentifier:SideCarIdentifier.to_string(),
					CommandIdentifier:CommandId.to_string(),
				},
			);
		}
	}

	fn UnregisterCommandInRegistry(&self, CommandId:&str) {
		if let Ok(mut Registry) = self
			.RunTime()
			.Environment
			.ApplicationState
			.Extension
			.Registry
			.CommandRegistry
			.lock()
		{
			Registry.remove(CommandId);
		}
	}
}
