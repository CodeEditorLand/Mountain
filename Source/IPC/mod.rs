//! # IPC Module
//! 
//! Contains the Mountain counterpart to Wind's IPC infrastructure.
//! Provides bidirectional communication between Mountain (Rust backend) and Wind (TypeScript frontend).

#![allow(non_snake_case, non_camel_case_types)]

pub mod TauriIPCServer;
pub mod WindServiceHandlers;
pub mod WindServiceAdapters;
pub mod ConfigurationBridge;
pub mod StatusReporter;
pub mod AdvancedFeatures;
pub mod WindAdvancedSync;

pub use TauriIPCServer::{
    TauriIPCServer,
    TauriIPCMessage,
    ConnectionStatus,
    mountain_ipc_receive_message,
    mountain_ipc_get_status,
};

pub use WindServiceHandlers::{
    mountain_ipc_invoke,
    register_wind_ipc_handlers,
};

pub use WindServiceAdapters::{
    WindServiceAdapter,
    WindDesktopConfiguration,
    WindEnvironmentService,
    WindFileService,
    WindStorageService,
    WindConfigurationService,
};

pub use ConfigurationBridge::{
    ConfigurationBridge,
    ConfigurationStatus,
    mountain_get_wind_desktop_configuration,
    mountain_update_configuration_from_wind,
    mountain_synchronize_configuration,
    mountain_get_configuration_status,
};

pub use StatusReporter::{
    StatusReporter,
    IPCStatusReport,
    mountain_get_ipc_status,
    mountain_get_ipc_status_history,
    mountain_start_ipc_status_reporting,
    initialize_status_reporter,
};

pub use AdvancedFeatures::{
    AdvancedFeatures,
    PerformanceStats,
    CollaborationSession,
    CollaborationPermissions,
    MessageCache,
    mountain_get_performance_stats,
    mountain_get_cache_stats,
    mountain_create_collaboration_session,
    mountain_get_collaboration_sessions,
    initialize_advanced_features,
};

pub use WindAdvancedSync::{
    WindAdvancedSync,
    DocumentSynchronization,
    SynchronizedDocument,
    DocumentChange,
    ChangeType,
    SyncState,
    SyncStatus,
    UIStateSynchronization,
    CursorPosition,
    SelectionRange,
    ViewState,
    LayoutState,
    RealTimeUpdates,
    RealTimeUpdate,
    UpdateType,
    mountain_add_document_for_sync,
    mountain_get_sync_status,
    mountain_subscribe_to_updates,
    initialize_wind_advanced_sync,
};
