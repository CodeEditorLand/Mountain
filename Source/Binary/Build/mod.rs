//! # Binary::Build
//!
//! Tauri builder and plugin configuration functions.
//! Each sub-module owns one plugin, one scheme handler, or one
//! command group, and exposes a single `Fn()` or `Plugin()` entry point
//! consumed by `Binary::Main::AppLifecycle`.

/// Construct and return the configured `tauri::Builder`.
pub mod TauriBuild;

/// Configure and create the primary application window.
pub mod WindowBuild;

/// Tauri plugin that routes all log events to the native trace sink.
pub mod LoggingPlugin;

/// Tauri plugin that proxies `localhost:` scheme requests to the IPC server.
pub mod LocalhostPlugin;

/// Custom URI scheme handlers for `vscode-file://`, `tauri://`, `land://`.
pub mod Scheme;

/// Service registry: maps service identifiers to their factory functions.
pub mod ServiceRegistry;

/// Tauri commands and resolver for DNS lookups.
pub mod DnsCommands;

/// TLS certificate lifecycle management (generation, renewal, trust store).
pub mod CertificateManager;

/// Tauri commands exposing TLS certificate status and manual rotation.
pub mod TlsCommands;

/// PostHog analytics plugin (opt-in telemetry event forwarding).
pub mod PostHogPlugin;

/// macOS app menu override - removes Undo/Redo so Cmd+Z reaches Monaco.
pub mod AppMenu;
