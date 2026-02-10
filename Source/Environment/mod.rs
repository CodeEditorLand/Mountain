//! # Environment Module
//!
//! ## RESPONSIBILITIES
//! Dependency Injection (DI) container that provides thread-safe access to
//! all Mountain providers through trait-based lookups using the Requires trait.
//!
//! ## ARCHITECTURAL ROLE
//!
//! The Environment module is the central dependency injection system for
//! Mountain:
//!
//! ```text
//! Component ──► Requires<T> ──► MountainEnvironment ──► Arc<dyn T>
//! ```
//!
//! ### Position in Mountain
//! - Implements Common crate's `Environment` and `Requires` traits
//! - All providers accessed through capability-based lookups
//! - Created early in startup and shared via `Arc<MountainEnvironment>`
//!
//! ### Key Components
//! - `MountainEnvironment`: Main DI container struct
//! - `ProviderTraitImplMacro`: Macro for generating trait implementations
//! - Provider modules: Individual implementations for each provider trait
//!
//! ### Provider Traits Implemented (25+)
//! - CommandExecutor, ConfigurationProvider, CustomEditorProvider
//! - DebugService, DiagnosticManager, DocumentProvider
//! - FileSystemReader/Writer, IPCProvider, KeybindingProvider
//! - LanguageFeatureProviderRegistry, OutputChannelManager
//! - SecretProvider, SourceControlManagementProvider
//! - StatusBarProvider, StorageProvider, SynchronizationProvider
//! - TerminalProvider, TestController, TreeViewProvider
//! - UserInterfaceProvider, WebviewProvider
//! - WorkspaceProvider, WorkspaceEditApplier
//! - ExtensionManagementService, SearchProvider
//!
//! ## ERROR HANDLING
//! Providers use CommonError for error reporting. The DI container handles
//! trait resolution at compile time, ensuring type safety.
//!
//! ## PERFORMANCE CONSIDERATIONS
//! - Thread-safe access via Arc<T>
//! - Lazy initialization through trait-based lookups
//! - Zero-cost abstractions - macro-generated code is identical to hand-written
//!
//! ## TODO
//! - [ ] Consider async initialization for providers
//! - [ ] Add provider health checking
//! - [ ] Implement provider dependency validation on initialization

// --- Main Environment Modules ---

/// Main DI container struct.
pub mod MountainEnvironment;

/// Macro for generating trait implementations.
pub mod ProviderTraitImplMacro;

// --- Provider Trait Implementations (organized by domain) ---
pub mod CommandProvider;

pub mod ConfigurationProvider;

pub mod CustomEditorProvider;

pub mod DebugProvider;

pub mod DiagnosticProvider;

pub mod DocumentProvider;

pub mod FileSystemProvider;

pub mod IPCProvider;

pub mod KeybindingProvider;

pub mod LanguageFeatureProvider;

pub mod OutputProvider;

pub mod SearchProvider;

pub mod SecretProvider;

pub mod SourceControlManagementProvider;

pub mod StatusBarProvider;

pub mod StorageProvider;

pub mod SynchronizationProvider;

pub mod TerminalProvider;

pub mod TestProvider;

pub mod TreeViewProvider;

pub mod UserInterfaceProvider;

// Re-export UserInterface and DTO for convenience
pub use CommonLibrary::UserInterface;

pub mod WebviewProvider;

pub mod WorkspaceProvider;

// --- Internal Utilities ---
// Shared helpers for provider implementations.
pub mod Utility;
