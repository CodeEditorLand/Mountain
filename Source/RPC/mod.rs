//! # RPC Service Module - Advanced Communication Layer
//!
//! Implements the gRPC service side of the Spine Contract for Mountain.
//! All services support feature flags (Debug, Development, Telemetry)
//! and include comprehensive OTEL instrumentation.
//!
//! ## Architecture
//!
//! ```
//! ┌──────────────────────────────────────────────────────────────┐
//! │                      RPC Module                              │
//! │  ┌────────────────────────────────────────────────────────┐ │
//! │  │                  gRPC Services                         │ │
//! │  │  • CommandService    - Command lifecycle               │ │
//! │  │  • WindowService     - Window/UI management             │ │
//! │  │  • WorkspaceService  - File/workspace operations        │ │
//! │  │  • SecretStorage     - Secure storage                  │ │
//! │  │  • CocoonService     - Extension host integration       │ │
//! │  └────────────────────────────────────────────────────────┘ │
//! │                          │                                   │
//! │  ┌────────────────────────────────────────────────────────┐ │
//! │  │                   State Managers                        │ │
//! │  │  • WindowState       - Window handle tracking          │ │
//! │  │  • SecretStorageState - Secret state management        │ │
//! │  └────────────────────────────────────────────────────────┘ │
//! │                          │                                   │
//! │  ┌────────────────────────────────────────────────────────┐ │
//! │  │               Telemetry & Metrics                       │ │
//! │  │  • OpenTelemetry tracing for all operations            │ │
//! │  │  • Custom metrics for performance monitoring          │ │
//! │  │  • Logging gates for build profiles                    │ │
//! │  └────────────────────────────────────────────────────────┘ │
//! └──────────────────────────────────────────────────────────────┘
//!                            │
//!                            ▼
//!              ┌──────────────────────────────┐
//!              │       MountainEnvironment     │
//!              │    (Capability Provider)      │
//!              └──────────────────────────────┘
//! ```
//!
//! ## Build Profiles
//!
//! - **Debug** (feature = "Debug"): Verbose logging, validation
//! - **Development** (feature = "Development"): Dev-friendly defaults
//! - **Telemetry** (feature = "Telemetry"): Full OTEL integration
//!
//! ## Code Style
//!
//! All modules follow:
//! - Extensive `//!` module documentation
//! - PascalCase, action-oriented function naming
//! - Structured logging with service prefixes
//! - Comprehensive error handling

// ============================================================================
// Service Implementations
// ============================================================================

pub mod CocoonService;
pub mod WindowService;
pub mod WorkspaceService;
pub mod CommandService;
pub mod SecretStorageService;

// ============================================================================
// State Management Modules
// ============================================================================

pub mod WindowState;
pub mod SecretStorageState;

// ============================================================================
// Re-exports for convenience
// ============================================================================

pub use CommandService::{
    CommandService,
    CommandMetadata,
    CommandStatistics,
    TelemetryConfig,
    LoggingGate,
    ServiceMetrics as CommandServiceMetrics,
};

pub use WindowService::{
    WindowService,
    WindowMetrics as WindowServiceMetrics,
};

pub use SecretStorageService::{
    SecretStorageService,
    SecretMetrics as SecretStorageServiceMetrics,
};

pub use WorkspaceService::{
    WorkspaceService,
    WorkspaceMetrics as WorkspaceServiceMetrics,
};

