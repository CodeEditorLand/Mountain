// File: Common/IpcDto.rs
// Defines Data Transfer Objects (DTOs) related to Inter-Process Communication,
// specifically for identifying RPC targets.

#![allow(non_snake_case, non_camel_case_types)]

// Defines the various RPC targets that can be invoked on either the
// MainThread (Mountain) or the ExtHost (Cocoon).
// This enum provides a type-safe way to construct RPC method names.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ProxyConfiguration {
	// MainThread (Mountain) Targets
	MainThreadCommands,
	MainThreadConfiguration,
	MainThreadDiagnostics,
	MainThreadDocuments,
	MainThreadExtensionEnablement,
	MainThreadFileSystem,
	MainThreadLanguageFeatures,
	MainThreadLanguages,
	MainThreadOutputService,
	MainThreadSecrets,
	MainThreadStorage,
	MainThreadTerminalService,
	MainThreadWindow,
	MainThreadWebviews,
	MainThreadTelemetry,
	MainThreadWorkspace,
	MainThreadStatusBar,

	// ExtHost (Cocoon) Targets
	ExtHostCommands,
	ExtHostConfiguration,
	ExtHostDiagnostics,
	ExtHostDocuments,
	ExtHostExtensionService,
	ExtHostFileSystemInfo,
	ExtHostLanguageFeatures,
	ExtHostLanguages,
	ExtHostOutputService,
	ExtHostStorage,
	ExtHostTerminalService,
	ExtHostEnv,
	ExtHostWebviews,
	ExtHostTelemetry,
	ExtHostChatProvider,
	ExtHostExtensionEnablement,
	ExtHostCustomEditors,
	ExtHostQuickInput,
	ExtHostMessageService,
	ExtHostDialogs,
	ExtHostAuthentication,
	ExtHostDebugService,
	ExtHostTaskService,
	ExtHostManagedSockets,
}

impl ProxyConfiguration {
	/// Returns the standardized string prefix for an RPC target.
	/// Example: `ProxyConfiguration::MainThreadCommands` ->
	/// `"MainThreadCommands"`
	pub fn GetTargetPrefix(&self) -> String { format!("{:?}", self) }
}
