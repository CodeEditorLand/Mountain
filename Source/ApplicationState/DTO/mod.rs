// @module DTO (ApplicationState)
// @description This module aggregates and re-exports all Data Transfer Objects
// (DTOs) that are used to represent the various components of the application's
// central state.
//

#![allow(non_snake_case, non_camel_case_types)]

// --- Module Declarations (alphabetical) ---
mod ConfigurationStateDto;
mod CustomDocumentStateDto;
mod DocumentStateDto;
mod ExtensionDescriptionStateDto;
mod HierarchySessionContextDto;
mod MarkerDataDto;
mod OutputChannelStateDto;
mod ProviderRegistrationDto;
mod RpcModelContentChangeDto;
mod TerminalStateDto;
mod TreeViewStateDto;
mod WebviewStateDto;
mod WindowStateDto;
mod WorkspaceFolderStateDto;

// --- Public Re-exports (alphabetical) ---
pub use self::{
	ConfigurationStateDto::MergedConfigurationStateDto,
	CustomDocumentStateDto::CustomDocumentStateDto,
	DocumentStateDto::DocumentStateDto,
	ExtensionDescriptionStateDto::ExtensionDescriptionStateDto,
	HierarchySessionContextDto::HierarchySessionContextDto,
	MarkerDataDto::MarkerDataDto,
	OutputChannelStateDto::OutputChannelStateDto,
	ProviderRegistrationDto::ProviderRegistrationDto,
	RpcModelContentChangeDto::{RpcModelContentChangeDto, RpcRangeDto},
	TerminalStateDto::TerminalStateDto,
	TreeViewStateDto::TreeViewStateDto,
	WebviewStateDto::WebviewStateDto,
	WindowStateDto::WindowStateDto,
	WorkspaceFolderStateDto::WorkspaceFolderStateDto,
};
