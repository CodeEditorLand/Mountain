
// This file declares the modules within the Handlers directory.
// Handlers are responsible for processing requests and events, often
// interacting with the AppState and Environment to perform their tasks.

pub mod Commands;
pub mod Config;
pub mod Diagnostics;
pub mod Documents;
pub mod Enablement;
pub mod ErrorUtils;
pub mod ExtensionStatus;
pub mod LanguageFeatures;
pub mod NativeFs; // Potentially to be reviewed/deprecated in favor of Environment::FilesystemProvider
pub mod Output;
pub mod ProcessManagement; // Renamed from process_mgmt
pub mod Protocol;
pub mod Proxy; // Potentially to be reviewed/deprecated if not used by gRPC
pub mod Registry; // Potentially for local handler registry, review if needed with gRPC
pub mod Secrets;
pub mod SkyCommands;
pub mod SkyConfiguration;
pub mod SkyDtos;
pub mod SkyIpcBridge; // Potentially to be reviewed/deprecated with direct gRPC
pub mod SkyUiResponses;
pub mod Storage;
pub mod Terminal;
pub mod Ui;
pub mod Workspace;
pub mod WorkspaceFsApi; // Potentially to be reviewed/deprecated in favor of Environment::FilesystemProvider
