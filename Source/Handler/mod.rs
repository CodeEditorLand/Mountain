// @module Handler
// @description This is the main aggregator for the entire business logic layer
// of the Mountain application.
//
// Each sub-module contains the concrete implementation logic for a specific
// feature or service contract defined in the `Common` crate. The `environment`
// providers delegate their work to the functions within these modules.

#![allow(non_snake_case, non_camel_case_types)]

// --- Core Service Handlers (alphabetical) ---
pub mod Command;
pub mod Config;
pub mod CustomEditor;
pub mod Diagnostic;
pub mod Document;
pub mod ExtensionManagement;
pub mod ExtensionStatus;
pub mod FileSystem;
pub mod LanguageFeature;
pub mod Output;
pub mod Secret;
pub mod StatusBar;
pub mod Storage;
pub mod Terminal;
pub mod TreeView;
pub mod UserInterface;
pub mod WebView;
pub mod WorkSpace;

// --- Internal and Bridge Handlers ---
pub mod ErrorUtility;
pub mod ProcessManagement;
pub mod Protocol;
pub mod SkyCommand;
pub mod SkyIPCBridge;
pub mod SkyUserInterfaceResponse;
