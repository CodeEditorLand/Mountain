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
/// Customdocumentstatedto module.
pub mod CustomDocumentStateDTO;

/// Documentstatedto module.
pub mod DocumentStateDTO;

/// Extensiondescriptionstatedto module.
pub mod ExtensionDescriptionStateDTO;

/// Markerdatadto module.
pub mod MarkerDataDTO;

/// Markerseverity module.
pub mod MarkerSeverity;

/// Mergedconfigurationstatedto module.
pub mod MergedConfigurationStateDTO;

/// Outputchannelstatedto module.
pub mod OutputChannelStateDTO;

/// Providerregistrationdto module.
pub mod ProviderRegistrationDTO;

/// Rpcrangedto module.
pub mod RPCRangeDTO;

/// Rpcmodelcontentchangedto module.
pub mod RPCModelContentChangeDTO;

/// Terminalstatedto module.
pub mod TerminalStateDTO;

/// Treeviewstatedto module.
pub mod TreeViewStateDTO;

/// Webviewstatedto module.
pub mod WebviewStateDTO;

/// Windowstatedto module.
pub mod WindowStateDTO;

/// Workspacefolderstatedto module.
pub mod WorkspaceFolderStateDTO;
