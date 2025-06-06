// File: AppState/mod.rs
// This module defines and exports the core application state structures.

#![allow(non_snake_case, non_camel_case_types)]

// Sub-modules for different parts of the application state and related helpers.
mod Analyze; // Helper for text analysis (e.g., line endings).
mod AppState; // The main AppState struct definition.
mod ConfigurationState; // DTO for merged configuration.
mod DocumentState; // DTO for a single open document.
mod ExtensionDescriptionState; // DTO for a scanned extension's metadata.
mod HierarchySessionContext; // Context for call/type hierarchy sessions.
mod Load; // Helper for loading Memento from disk.
mod OutputChannelState; // DTO for a single output channel.
mod ProviderRegistration; // DTO for a registered language feature provider.
mod Resolve; // Helper for resolving Memento file paths.
mod RpcModelContentChange; // DTO for RPC-based document content changes.
mod TerminalState; // DTO for a single active terminal instance.
mod UrlSerdeHelper; // Serde helper for `url::Url`.
mod WorkspaceFolderState; // DTO for a single workspace folder.

// Re-export the primary AppState struct.
pub use self::AppState::AppState;
