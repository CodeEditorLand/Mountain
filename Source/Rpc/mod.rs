// File: Rpc/mod.rs
// This module defines the RPC (Remote Procedure Call) interface and handlers
// for communication, likely between the Mountain backend and the Cocoon
// sidecar. With the introduction of gRPC, this module will evolve to define
// gRPC service handlers and related DTOs (Data Transfer Objects).

// Sub-module for argument DTOs used in RPC calls.
pub mod Argument;

// Main RPC handler structs. These will likely become gRPC service
// implementations.
mod MainThreadCommandsHandler;
mod MainThreadConfigurationHandler;
mod MainThreadDiagnosticsHandler;
mod MainThreadDialogsHandler;
mod MainThreadDocumentsHandler;
mod MainThreadExtensionEnablementHandler; // Added based on DTOs
mod MainThreadExtensionServiceHandler;
mod MainThreadFileSystemApiHandler;
mod MainThreadLanguageFeaturesHandler; // This will be the new name for LanguageFeatures
mod MainThreadLogHandler;
mod MainThreadMessageHandler;
mod MainThreadOutputServiceHandler;
mod MainThreadSecretsHandler;
mod MainThreadStatusBarHandler;
mod MainThreadStorageHandler;
mod MainThreadTerminalServiceHandler;
mod MainThreadWindowHandler;
mod MainThreadWorkspaceHandler;

// Re-exporting the handlers for easier access from other modules.
pub use self::MainThreadExtensionEnablementHandler::MainThreadExtensionEnablementHandler; // Added
pub use self::MainThreadLanguageFeaturesHandler::MainThreadLanguageFeaturesHandler; // New Name
pub use self::{
	MainThreadCommandsHandler::MainThreadCommandsHandler,
	MainThreadConfigurationHandler::MainThreadConfigurationHandler,
	MainThreadDiagnosticsHandler::MainThreadDiagnosticsHandler,
	MainThreadDialogsHandler::MainThreadDialogsHandler,
	MainThreadDocumentsHandler::MainThreadDocumentsHandler,
	MainThreadExtensionServiceHandler::MainThreadExtensionServiceHandler,
	MainThreadFileSystemApiHandler::MainThreadFileSystemApiHandler,
	MainThreadLogHandler::MainThreadLogHandler,
	MainThreadMessageHandler::MainThreadMessageHandler,
	MainThreadOutputServiceHandler::MainThreadOutputServiceHandler,
	MainThreadSecretsHandler::MainThreadSecretsHandler,
	MainThreadStatusBarHandler::MainThreadStatusBarHandler,
	MainThreadStorageHandler::MainThreadStorageHandler,
	MainThreadTerminalServiceHandler::MainThreadTerminalServiceHandler,
	MainThreadWindowHandler::MainThreadWindowHandler,
	MainThreadWorkspaceHandler::MainThreadWorkspaceHandler,
};

// Setup function for the RPC server (likely gRPC server setup).
mod Setup;
pub use Setup::SetupMountainRpcServer;
