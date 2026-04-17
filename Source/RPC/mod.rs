//! # Mountain RPC Services
//!
//!  ☀️ 🟢 MOUNTAIN_SKY_ONLY - Core RPC service implementations

#![allow(unused_imports, unused_variables)]
//! This module contains the complete RPC services for Mountain's Spine
//! contract. All services support extension hosts based on their feature gates:
//!
//! ## Service Classification by Support Level
//!
//! ### 🟢 GREEN - Full Support (All Hosts)
//! - **EchoAction**: Bidirectional actions, host registration, routing
//! - **Commands**: Command registration and execution
//! - **Workspace**: File operations, document management
//! - **Configuration**: Configuration read/write
//!
//! ### 🟡 YELLOW - Partial Support (Grove, Cocoon)
//! - **Windows**: Webviews, documents (limited in Sky)
//! - **Tree Views**: Tree data providers (read-only in Sky)
//! - **Language Features**: Completion, diagnostics (basic in Sky)
//!
//! ### 🔴 RED - Cocoon Only Services
//! - **Terminals**: Terminal emulation and pseudo-terminals
//! - **Debug**: Debug adapter protocol integration
//! - **SCM**: Source control management (git)
//! - **Processes**: Child process execution
//!
//! ### 🔵 BLUE - WASM Optimized
//! - **Document Operations**: Zero-copy memory access in WASM
//! - **File Operations**: Parallel search in WASM
//!
//! ## Module Structure
//!
//! Services are split into atomic submodules for granular feature gates:
//!
//! ```text
//! RPC/
//! ├── EchoAction/ # ☀️ 🟢 Central EchoAction system
//! ├── Commands/ # ☀️ 🟢 Command registration
//! │ └── Validation/ # Input validation
//! ├── Workspace/ # ☀️ 🟢 File/workspace operations
//! ├── Configuration/ # ☀️ 🟢 Configuration management
//! ├── Windows/ # ☀️ 🟡 Window and document services
//! ├── Terminals/ # ☀️ 🔴 Terminal services (Cocoon only)
//! ├── Debug/ # ☀️ 🔴 Debug protocol (Cocoon only)
//! ├── SCM/ # ☀️ 🔴 Source control (Cocoon only)
//! ├── Processes/ # ☀️ 🔴 Child processes (Cocoon only)
//! ├── Telemetry/ # OTEL integration
//! │ ├── Spans/ # Span management
//! │ └── Metrics/ # Metrics recording
//! └── types/ # Shared types
//! ```

pub mod CocoonService;
pub use CocoonService::CocoonServiceImpl;

#[path = "Types.rs"]
pub mod Types;

#[path = "EchoAction.rs"]
pub mod EchoAction;
pub use EchoAction::{EchoActionServer, ExtensionHostRegistry, ExtensionRouter};

#[path = "Commands.rs"]
pub mod Commands;
pub use Commands::{CommandService, CommandValidation};

#[path = "Workspace.rs"]
pub mod Workspace;
pub use Workspace::WorkspaceService;

#[path = "Configuration.rs"]
pub mod Configuration;
pub use Configuration::ConfigurationService;

// Conditionally include services based on feature flags

#[cfg(any(feature = "grove", feature = "cocoon"))]
#[path = "Windows.rs"]
pub mod Windows;
#[cfg(any(feature = "grove", feature = "cocoon"))]
pub use Windows::WindowService;

#[cfg(feature = "terminals")]
#[path = "Terminals.rs"]
pub mod Terminals;
#[cfg(feature = "terminals")]
pub use Terminals::TerminalService;

#[cfg(feature = "debug-protocol")]
#[path = "Debug.rs"]
pub mod Debug;
#[cfg(feature = "debug-protocol")]
pub use Debug::DebugService;

#[cfg(feature = "scm-support")]
#[path = "SCM.rs"]
pub mod SCM;
#[cfg(feature = "scm-support")]
pub use SCM::SCMService;

#[cfg(feature = "child-processes")]
#[path = "Processes.rs"]
pub mod Processes;
#[cfg(feature = "child-processes")]
pub use Processes::ProcessService;

// Telemetry modules
#[path = "Telemetry.rs"]
pub mod Telemetry;
pub use Telemetry::{TelemetryService, metrics::ServiceMetrics, spans::TraceSpan};

// Re-export vine types
#[path = "Vine.rs"]
pub mod Vine;
