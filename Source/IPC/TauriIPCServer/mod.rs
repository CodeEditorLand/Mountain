
//! New (atomic) Tauri IPC server orchestrator. The legacy entry point lives
//! in `IPC::TauriIPCServer_Old`; consumers migrate to
//! `IPC::TauriIPCServer::Server::TauriIPCServer` as the rewrite lands.

pub mod Server;
