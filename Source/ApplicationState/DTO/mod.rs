

/**
 * @module Dto (AppState)
 * @description This module aggregates and re-exports all Data Transfer Objects (DTOs)
 * that are used to represent the various components of the application's central state.
 */

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
pub use self::ConfigurationStateDto::MergedConfigurationStateDto;
pub use self::CustomDocumentStateDto::CustomDocumentStateDto;
pub use self::DocumentStateDto::DocumentStateDto;
pub use self::ExtensionDescriptionStateDto::ExtensionDescriptionStateDto;
pub use self::HierarchySessionContextDto::HierarchySessionContextDto;
pub use self::MarkerDataDto::MarkerDataDto;
pub use self::OutputChannelStateDto::OutputChannelStateDto;
pub use self::ProviderRegistrationDto::ProviderRegistrationDto;
pub use self::RpcModelContentChangeDto::{RpcModelContentChangeDto, RpcRangeDto};
pub use self::TerminalStateDto::TerminalStateDto;
pub use self::TreeViewStateDto::TreeViewStateDto;
pub use self::WebviewStateDto::WebviewStateDto;
pub use self::WindowStateDto::WindowStateDto;
pub use self::WorkspaceFolderStateDto::WorkspaceFolderStateDto;
