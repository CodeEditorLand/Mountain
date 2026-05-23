
//! # Binary::IPC
//!
//! All `#[tauri::command]` handlers exposed to the frontend invoke system.
//! Each sub-module owns exactly one command function and its supporting
//! types, keeping the Tauri invoke handler registration flat and auditable.
//!
//! Commands are async and do not block the main thread. All follow the
//! Tauri security model; no command bypasses the invoke allow-list.

/// Return the current workbench configuration as JSON.
pub mod WorkbenchConfigurationCommand;

/// Receive a pending IPC message from the backend queue.
pub mod MessageReceiveCommand;

/// Get the current application status snapshot.
pub mod StatusGetCommand;

/// Forward a generic invoke payload to the internal IPC router.
pub mod InvokeCommand;

/// Read or write the Wind desktop configuration.
pub mod WindConfigurationCommand;

/// Apply a configuration key-value update.
pub mod ConfigurationUpdateCommand;

/// Trigger a configuration sync to disk.
pub mod ConfigurationSyncCommand;

/// Return the current configuration load/validation status.
pub mod ConfigurationStatusCommand;

/// Return a full configuration data snapshot.
pub mod ConfigurationDataCommand;

/// Return the current IPC channel status.
pub mod IPCStatusCommand;

/// Return the IPC status history ring buffer.
pub mod IPCStatusHistoryCommand;

/// Start periodic IPC status reporting.
pub mod IPCStatusReportingStartCommand;

/// Return a performance statistics snapshot.
pub mod PerformanceStatsCommand;

/// Start or query a collaboration session.
pub mod CollaborationSessionCommand;

/// Sync a document state payload from the frontend.
pub mod DocumentSyncCommand;

/// Subscribe to or unsubscribe from update notifications.
pub mod UpdateSubscriptionCommand;

/// Return asset and path-canon cache occupancy statistics.
pub mod CacheStatsCommand;

/// Spawn or signal a managed child process.
pub mod ProcessCommand;

/// Return a liveness and readiness health check payload.
pub mod HealthCommand;

/// Add, remove, or list workspace folder entries.
pub mod WorkspaceFolderCommand;

/// Forward a renderer dev-log entry to the native trace sink.
pub mod RenderDevLogCommand;

// LAND-PATCH B7-S6 P14.5: Vine notification broadcast subscription
// surface for Sky/Wind.
/// Subscribe to or unsubscribe from Vine notification broadcasts.
pub mod VineSubscribeCommand;
