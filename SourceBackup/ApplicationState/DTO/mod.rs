// @module DTO (ApplicationState)
// @description This module aggregates and re-exports all Data Transfer Objects
// (DTOs) that are used to represent the various components of the application's
// central state.
//

#![allow(non_snake_case, non_camel_case_types)]

// --- Module Declarations (alphabetical) ---
mod CustomDocumentStateDTO;
mod DocumentStateDTO;
mod ExtensionDescriptionStateDTO;
mod MarkerDataDTO;
mod MergedConfigurationStateDTO;
mod OutputChannelStateDTO;
mod ProviderRegistrationDTO;
mod RPCModelContentChangeDTO;
mod TerminalStateDTO;
mod TreeViewStateDTO;
mod WebViewStateDTO;
mod WindowStateDTO;
mod WorkspaceFolderStateDTO;

// --- Public Re-exports (alphabetical) ---
pub use self::{
	CustomDocumentStateDTO::CustomDocumentStateDTO,
	DocumentStateDTO::DocumentStateDTO,
	ExtensionDescriptionStateDTO::ExtensionDescriptionStateDTO,
	MarkerDataDTO::MarkerDataDTO,
	MergedConfigurationStateDTO::MergedConfigurationStateDTO,
	OutputChannelStateDTO::OutputChannelStateDTO,
	ProviderRegistrationDTO::ProviderRegistrationDTO,
	RPCModelContentChangeDTO::{RPCModelContentChangeDTO, RPCRangeDTO},
	TerminalStateDTO::TerminalStateDTO,
	TreeViewStateDTO::TreeViewStateDTO,
	WebViewStateDTO::WebViewStateDTO,
	WindowStateDTO::WindowStateDTO,
	WorkspaceFolderStateDTO::WorkspaceFolderStateDTO,
};
