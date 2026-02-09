//! # Mountain RPC Services
//!
//! ☀️ 🟢 MOUNTAIN_SKY_ONLY - Core RPC service implementations
//! 
//! This module contains the complete RPC services for Mountain's Spine contract.
//! All services support extension hosts based on their feature gates:
//!
//! ## Service Classification by Support Level
//!
//! ### 🟢 GREEN - Full Support (All Hosts)
//! - **EchoAction**: Bidirectional actions, host registration, routing
//! - **Commands**: Command registration and execution
//! - **Workspace**: File operations, document management
//! - **Configuration**: Configuration read/write
//!
//! ### 🟡 YELLOW - Partial Support (Grove, Cocoon)
//! - **Windows**: Webviews, documents (limited in Sky)
//! - **Tree Views**: Tree data providers (read-only in Sky)
//! - **Language Features**: Completion, diagnostics (basic in Sky)
//!
//! ### 🔴 RED - Cocoon Only Services
//! - **Terminals**: Terminal emulation and pseudo-terminals
//! - **Debug**: Debug adapter protocol integration
//! - **SCM**: Source control management (git)
//! - **Processes**: Child process execution
//!
//! ### 🔵 BLUE - WASM Optimized
//! - **Document Operations**: Zero-copy memory access in WASM
//! - **File Operations**: Parallel search in WASM
//!
//! ## Module Structure
//!
//! Services are split into atomic submodules for granular feature gates:
//!
//! ```
//! RPC/
//! ├── EchoAction/           # ☀️ 🟢 Central EchoAction system
//! ├── Commands/             # ☀️ 🟢 Command registration
//! │   └── Validation/       # Input validation
//! ├── Workspace/            # ☀️ 🟢 File/workspace operations
//! ├── Configuration/        # ☀️ 🟢 Configuration management
//! ├── Windows/              # ☀️ 🟡 Window and document services
//! ├── Terminals/            # ☀️ 🔴 Terminal services (Cocoon only)
//! ├── Debug/                # ☀️ 🔴 Debug protocol (Cocoon only)
//! ├── SCM/                  # ☀️ 🔴 Source control (Cocoon only)
//! ├── Processes/            # ☀️ 🔴 Child processes (Cocoon only)
//! ├── Telemetry/            # OTEL integration
//! │   ├── Spans/            # Span management
//! │   └── Metrics/          # Metrics recording
//! └── types/                # Shared types
//! ```

pub mod CocoonService;
pub use CocoonService::CocoonServiceImpl;

pub mod types;

pub mod echo_action;
pub use echo_action::{EchoActionServer, ExtensionHostRegistry, ExtensionRouter};

pub mod commands;
pub use commands::{CommandService, CommandValidation};

pub mod workspace;
pub use workspace::WorkspaceService;

pub mod configuration;
pub use configuration::ConfigurationService;

// Conditionally include services based on feature flags

#[cfg(any(feature = "grove", feature = "cocoon"))]
pub mod windows;
#[cfg(any(feature = "grove", feature = "cocoon"))]
pub use windows::WindowService;

#[cfg(feature = "terminals")]
pub mod terminals;
#[cfg(feature = "terminals")]
pub use terminals::TerminalService;

#[cfg(feature = "debug-protocol")]
pub mod debug;
#[cfg(feature = "debug-protocol")]
pub use debug::DebugService;

#[cfg(feature = "scm-support")]
pub mod scm;
#[cfg(feature = "scm-support")]
pub use scm::SCMService;

#[cfg(feature = "child-processes")]
pub mod processes;
#[cfg(feature = "child-processes")]
pub use processes::ProcessService;

// Telemetry modules
pub mod telemetry;
pub use telemetry::TelemetryService;
pub use telemetry::spans::TraceSpan;
pub use telemetry::metrics::ServiceMetrics;

// Re-export vine types
pub mod vine;

