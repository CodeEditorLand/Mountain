//! # ApplicationState DTO Module
//!
//! # RESPONSIBILITY
//! - Aggregates and re-exports all Data Transfer Objects (DTOs)
//! - Central module for state serialization/deserialization
//! - Provides standard interface for gRPC/IPC transmission of application state
//!
//! # FIELDS
//! - Re-exports all DTO modules for application state components

// --- Module Declarations (alphabetical) ---
pub mod CustomDocumentStateDTO;
pub mod DocumentStateDTO;
pub mod ExtensionDescriptionStateDTO;
pub mod MarkerDataDTO;
pub mod MarkerSeverity;
pub mod MergedConfigurationStateDTO;
pub mod OutputChannelStateDTO;
pub mod ProviderRegistrationDTO;
pub mod RPCRangeDTO;
pub mod RPCModelContentChangeDTO;
pub mod TerminalStateDTO;
pub mod TreeViewStateDTO;
pub mod WebviewStateDTO;
pub mod WindowStateDTO;
pub mod WorkspaceFolderStateDTO;
