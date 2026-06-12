//! Serializable DTOs used for state persistence and IPC transmission.
//!
//! Each DTO module follows the convention: one file per type, `pub struct
//! Struct` as the primary export, matching the Cocoon wire format.
//!
//! ## Sub-modules
//!
//! - [`CustomDocumentStateDTO`]: Custom document state
//! - [`DocumentStateDTO`]: Standard document state
//! - [`ExtensionDescriptionStateDTO`]: Extension description state
//! - [`MarkerDataDTO`]: Marker (diagnostic) data
//! - [`MarkerSeverity`]: Marker severity levels
//! - [`MergedConfigurationStateDTO`]: Merged configuration state
//! - [`OutputChannelStateDTO`]: Output channel state
//! - [`ProviderRegistrationDTO`]: Provider registration state
//! - [`RPCRangeDTO`]: LSP-compatible range DTO
//! - [`RPCModelContentChangeDTO`]: Model content change DTO
//! - [`TerminalStateDTO`]: Terminal instance state
//! - [`TreeViewStateDTO`]: Tree view panel state
//! - [`WebviewStateDTO`]: Webview panel state
//! - [`WindowStateDTO`]: Window state
//! - [`WorkspaceFolderStateDTO`]: Workspace folder state

/// Custom document state DTO.
pub mod CustomDocumentStateDTO;

/// Standard document state DTO.
pub mod DocumentStateDTO;

/// Extension description state DTO.
pub mod ExtensionDescriptionStateDTO;

/// Marker (diagnostic) data DTO.
pub mod MarkerDataDTO;

/// Marker severity levels enumeration.
pub mod MarkerSeverity;

/// Merged configuration state DTO.
pub mod MergedConfigurationStateDTO;

/// Output channel state DTO.
pub mod OutputChannelStateDTO;

/// Provider registration state DTO.
pub mod ProviderRegistrationDTO;

/// LSP-compatible range DTO for IPC transmission.
pub mod RPCRangeDTO;

/// Model content change DTO for IPC transmission.
pub mod RPCModelContentChangeDTO;

/// Terminal instance state DTO.
pub mod TerminalStateDTO;

/// Tree view panel state DTO.
pub mod TreeViewStateDTO;

/// Webview panel state DTO.
pub mod WebviewStateDTO;

/// Window state DTO.
pub mod WindowStateDTO;

/// Workspace folder state DTO.
pub mod WorkspaceFolderStateDTO;
