// File: Rpc/MainThreadLanguageFeaturesHandler.rs
// Defines the RPC handler for language feature provider registrations and
// event emissions from the sidecar. This is the new, more specific name for
// this handler.

use std::sync::Arc;

use Common::LanguageFeatureEffects::ProviderType as CommonLanguageProviderType;
use log::{debug, info, warn};
use serde_json::{Value, json};
use tauri::{AppHandle, Manager, State, Wry};

use crate::Runtime::AppRuntime; // May not be strictly needed if logic is in Handlers
use crate::{
	Handlers::{self, ErrorUtils, LanguageFeatures as LanguageFeaturesHandlerLogic},
	Rpc::Args::LanguageFeatures::{EmitProviderEventArgument, RegisterProviderArgument, UnregisterProviderArgument},
};

#[derive(Clone)]
pub struct MainThreadLanguageFeaturesHandler {
	pub ApplicationHandle:AppHandle<Wry>,
	// Runtime might not be needed if all logic is in Handlers::LanguageFeatures
	// pub Runtime: Arc<AppRuntime>,
}

impl MainThreadLanguageFeaturesHandler {
	pub fn New(ApplicationHandle:AppHandle<Wry> /* , Runtime: Arc<AppRuntime> */) -> Self {
		Self { ApplicationHandle /* , Runtime */ }
	}

	/// Registers a language feature provider of a generic type.
	/// The specific provider type is determined by the `ActualProviderType`
	/// argument.
	pub async fn RegisterProviderGeneric(
		&self,
		SidecarIdentifier:&str,
		ActualProviderType:CommonLanguageProviderType,
		Argument:RegisterProviderArgument,
	) -> Result<Value, String> {
		let ExtensionIdentifierString = Argument
			.ExtensionIdentifierDto
			.get("value")
			.and_then(Value::as_str)
			.unwrap_or("unknown_extension_for_generic_registration");

		info!(
			"[Rpc LanguageFeaturesHandler] RegisterProviderGeneric (DTO): Type='{:?}', CocoonHandle={}, \
			 Extension='{}', Sidecar='{}'",
			ActualProviderType, Argument.Handle, ExtensionIdentifierString, SidecarIdentifier
		);

		// The options_dto from RegisterProviderArgument is a generic Value.
		// It needs to be parsed into the specific ProviderOptionsDto variant if
		// applicable. This logic is adapted from the `track.rs` "improvements"
		// section.
		let ParsedSpecificOptionsDto = LanguageFeaturesHandlerLogic::ParseProviderOptionsForType(
			ActualProviderType,
			Argument.OptionsDto.as_ref(),
			ExtensionIdentifierString,
			Argument.Handle,
		);

		LanguageFeaturesHandlerLogic::RegisterProviderInAppState(
			&self.ApplicationHandle,
			SidecarIdentifier,
			Argument.Handle,
			ActualProviderType,
			Argument.SelectorDto,
			ParsedSpecificOptionsDto,
			Argument.ExtensionIdentifierDto,
		)
		.await
	}

	/// Unregisters a previously registered language feature provider.
	pub async fn UnregisterProvider(
		&self,
		SidecarIdentifier:&str,
		Argument:UnregisterProviderArgument,
	) -> Result<Value, String> {
		info!(
			"[Rpc LanguageFeaturesHandler] UnregisterProvider (DTO): Handle={}, Sidecar='{}'",
			Argument.Handle, SidecarIdentifier
		);
		LanguageFeaturesHandlerLogic::UnregisterProviderFromAppState(
			&self.ApplicationHandle,
			SidecarIdentifier,
			Argument.Handle,
		)
		.await
	}

	/// Handles an event emission from a provider in the sidecar (e.g.,
	/// onDidChangeCodeLenses).
	pub async fn EmitProviderEvent(
		&self,
		SidecarIdentifier:&str,
		EventMethodName:String, // e.g., "emitCodeLensEvent", "emitInlayHintsEvent"
		Argument:EmitProviderEventArgument,
	) -> Result<Value, String> {
		info!(
			"[Rpc LanguageFeaturesHandler] EmitProviderEvent (DTO): Sidecar='{}', EventMethod='{}', Handle={}, \
			 ArgsIsSome={}",
			SidecarIdentifier,
			EventMethodName,
			Argument.EventHandle,
			Argument.EventArguments.is_some()
		);
		LanguageFeaturesHandlerLogic::HandleProviderEventEmission(
			&self.ApplicationHandle,
			EventMethodName,
			Argument.EventHandle,
			Argument.EventArguments,
		)
		.await
	}
}
