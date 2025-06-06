// File: Rpc/MainThreadOutputServiceHandler.rs
// Defines the RPC handler for output channel operations requested by the
// sidecar. This includes registering, appending to, clearing, revealing,
// closing, and disposing of output channels.

use std::sync::Arc;

use Common::{Errors::CommonError, OutputEffects, Runtime::AppRuntimeTrait};
use log::{debug, info, trace};
use serde_json::{Value, json};
use tauri::{AppHandle, Manager, State, Wry};

use crate::Rpc::Args::Output::{
	AppendArgument as AppendToOutputChannelArgument,
	OutputChannelIdentifierArgument, // For Clear, Close, Dispose
	RegisterOutputChannelArgument,
	ReplaceArgument as ReplaceOutputChannelArgument,
	RevealArgument as RevealOutputChannelArgument,
};
use crate::{Handlers::ErrorUtils, Runtime::AppRuntime};

#[derive(Clone)]
pub struct MainThreadOutputServiceHandler {
	pub ApplicationHandle:AppHandle<Wry>,
	pub Runtime:Arc<AppRuntime>,
}

impl MainThreadOutputServiceHandler {
	pub fn New(ApplicationHandle:AppHandle<Wry>, Runtime:Arc<AppRuntime>) -> Self {
		Self { ApplicationHandle, Runtime }
	}

	/// Registers a new output channel.
	pub async fn Register(&self, Argument:RegisterOutputChannelArgument) -> Result<Value, String> {
		info!(
			"[Rpc OutputServiceHandler] Register (DTO): Name='{}', LanguageIdentifier='{:?}'",
			Argument.Name, Argument.LanguageIdentifier
		);
		// The effect expects direct parameters, not a DTO.
		let Effect = OutputEffects::RegisterOutputChannel(Argument.Name, Argument.LanguageIdentifier);
		self.Runtime
			.Run(Effect)
			.await
			.map(|ChannelIdentifierString| json!(ChannelIdentifierString))
			.map_err(|Error| ErrorUtils::MapCommonErrorToRpcString(Error, "RegisterOutputChannel DTO"))
	}

	/// Appends text to an existing output channel.
	pub async fn Append(&self, Argument:AppendToOutputChannelArgument) -> Result<Value, String> {
		trace!(
			"[Rpc OutputServiceHandler] Append (DTO): ChannelIdentifier='{}', ContentLength={}",
			Argument.ChannelIdentifier,
			Argument.Content.len()
		);
		let Effect = OutputEffects::AppendToOutputChannel(Argument.ChannelIdentifier, Argument.Content);
		self.Runtime
			.Run(Effect)
			.await
			.map(|_| Value::Null)
			.map_err(|Error| ErrorUtils::MapCommonErrorToRpcString(Error, "AppendToOutputChannel DTO"))
	}

	/// Clears the content of an existing output channel.
	pub async fn Clear(&self, Argument:OutputChannelIdentifierArgument) -> Result<Value, String> {
		info!(
			"[Rpc OutputServiceHandler] Clear (DTO): ChannelIdentifier='{}'",
			Argument.ChannelIdentifier
		);
		let Effect = OutputEffects::ClearOutputChannel(Argument.ChannelIdentifier);
		self.Runtime
			.Run(Effect)
			.await
			.map(|_| Value::Null)
			.map_err(|Error| ErrorUtils::MapCommonErrorToRpcString(Error, "ClearOutputChannel DTO"))
	}

	/// Replaces the entire content of an existing output channel.
	pub async fn Replace(&self, Argument:ReplaceOutputChannelArgument) -> Result<Value, String> {
		info!(
			"[Rpc OutputServiceHandler] Replace (DTO): ChannelIdentifier='{}', NewContentLength={}",
			Argument.ChannelIdentifier,
			Argument.Content.len()
		);
		let Effect = OutputEffects::ReplaceOutputChannelContent(Argument.ChannelIdentifier, Argument.Content);
		self.Runtime
			.Run(Effect)
			.await
			.map(|_| Value::Null)
			.map_err(|Error| ErrorUtils::MapCommonErrorToRpcString(Error, "ReplaceOutputChannelContent DTO"))
	}

	/// Reveals (shows) an output channel to the user.
	pub async fn Reveal(&self, Argument:RevealOutputChannelArgument) -> Result<Value, String> {
		info!(
			"[Rpc OutputServiceHandler] Reveal (DTO): ChannelIdentifier='{}', PreserveFocus={:?}",
			Argument.ChannelIdentifier, Argument.PreserveFocus
		);
		// The ViewColumn aspect from the DTO is not directly used in the simple effect
		// signature. If view column matters, the effect or its environment
		// implementation would need to handle it.
		let Effect =
			OutputEffects::RevealOutputChannel(Argument.ChannelIdentifier, Argument.PreserveFocus.unwrap_or(false));
		self.Runtime
			.Run(Effect)
			.await
			.map(|_| Value::Null)
			.map_err(|Error| ErrorUtils::MapCommonErrorToRpcString(Error, "RevealOutputChannel DTO"))
	}

	/// Closes the view of an output channel (does not dispose it).
	pub async fn Close(&self, Argument:OutputChannelIdentifierArgument) -> Result<Value, String> {
		info!(
			"[Rpc OutputServiceHandler] Close (DTO): ChannelIdentifier='{}'",
			Argument.ChannelIdentifier
		);
		let Effect = OutputEffects::CloseOutputChannelView(Argument.ChannelIdentifier);
		self.Runtime
			.Run(Effect)
			.await
			.map(|_| Value::Null)
			.map_err(|Error| ErrorUtils::MapCommonErrorToRpcString(Error, "CloseOutputChannelView DTO"))
	}

	/// Disposes of an output channel, removing it completely.
	pub async fn Dispose(&self, Argument:OutputChannelIdentifierArgument) -> Result<Value, String> {
		info!(
			"[Rpc OutputServiceHandler] Dispose (DTO): ChannelIdentifier='{}'",
			Argument.ChannelIdentifier
		);
		let Effect = OutputEffects::DisposeOutputChannel(Argument.ChannelIdentifier);
		self.Runtime
			.Run(Effect)
			.await
			.map(|_| Value::Null)
			.map_err(|Error| ErrorUtils::MapCommonErrorToRpcString(Error, "DisposeOutputChannel DTO"))
	}
}
