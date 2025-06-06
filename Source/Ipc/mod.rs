

// File: Ipc/mod.rs
// Declares and exports modules for the IPC (Inter-Process Communication) system.

#![allow(non_snake_case, non_camel_case_types)] 

// Sub-modules for different components of the IPC system.
mod Manager;
mod MessageDispatcher;
mod RpcProtocolAdapter;

// Sub-module for gRPC client components.
pub mod GrpcClient {
    // Re-exporting modules from a flattened directory structure.
    pub use super::super::GrpcClient::*;
}

// Sub-module for gRPC server components.
pub mod GrpcServer {
    pub use super::super::GrpcServer::*;
}

// Sub-module for utility functions, like Protobuf value converters.
pub mod Util {
    pub use super::super::Util::*;
}

// Sub-module for generated types (e.g., from .proto files).
pub mod Generated {
    pub mod VineGrpcPb {
        pub use super::super::VineGrpcPb::*;
    }
}

// Re-exporting the primary public functions and types from the Manager module
// with PascalCase naming for direct use by other parts of the application.
pub use self::Manager::{
    Initialize,
    IsInitialized,
    Shutdown,
    getIPC as GetIpc, // Renamed for clarity
    skyToCocoonMessageBus as EventBus, // Renamed for clarity
    OnConfigurationChanged,
    OnWorkspaceFoldersChanged,
    InitializeIpcCancellation,
    GetCancellationTokenRegistry,
    RegisterCocoonInvokeHandler,
    type CocoonPrimaryIpcApi as PrimaryIpcApi, // Renamed type alias
    type ConfigurationChangedEventPayload as ConfigurationEvent, // Renamed type alias
    type WorkspaceFoldersChangedEventPayload as WorkspaceFoldersEvent, // Renamed type alias
};

// Re-exporting core dispatcher functions.
pub use self::MessageDispatcher::{Initialize as InitializeMessageDispatcher, Dispatch as DispatchMessage};
