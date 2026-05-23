//! Shared IPC abstractions used across `IPC/`. Each submodule owns one
//! concept; callers spell the full path (`IPC::Common::HealthStatus::Foo`).

pub mod ConnectionStatus;

pub mod HealthStatus;

pub mod MessageType;

pub mod PerformanceMetrics;

pub mod ServiceInfo;
