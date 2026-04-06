//! # FeatureState Module (ApplicationState)
//!
//! ## RESPONSIBILITIES
//! Manages feature-specific state including diagnostics, documents, terminals,
//! webviews, tree views, output channels, and markers.
//!
//! ## ARCHITECTURAL ROLE
//! FeatureState is part of the **state organization layer**, representing
//! all feature-specific state in the application.
//!
//! ## KEY COMPONENTS
//! - Diagnostics: Diagnostic errors state
//! - Documents: Open documents state
//! - Terminals: Terminal instances state
//! - Webviews: Webview panels state
//! - TreeViews: Tree view providers state
//! - OutputChannels: Output channel state
//! - Markers: Marker state
//! - State: Main struct combining all feature state
//!
//! ## ERROR HANDLING
//! Uses `Arc<Mutex<...>>` for thread-safe access with proper error handling.
//!
//! ## LOGGING
//! State changes are logged at appropriate levels.
//!
//! ## PERFORMANCE CONSIDERATIONS
//! - Lock mutexes briefly
//! - Avoid nested locks
//! - Use Arc for shared ownership
//!
//! ## TODO
//! - [ ] Add feature state validation
//! - [ ] Implement feature lifecycle events
//! - [ ] Add feature metrics

pub mod Debug;
pub mod Decorations;
pub mod Diagnostics;
pub mod Documents;
pub mod Keybindings;
pub mod LifecyclePhase;
pub mod Markers;
pub mod OutputChannels;
pub mod Terminals;
pub mod TreeViews;
pub mod Webviews;
pub mod WorkingCopy;
pub mod State;

pub use Debug::*;
pub use Decorations::*;
pub use Diagnostics::*;
pub use Documents::*;
pub use Keybindings::*;
pub use LifecyclePhase::*;
pub use Markers::*;
pub use OutputChannels::*;
pub use Terminals::*;
pub use TreeViews::*;
pub use Webviews::*;
pub use WorkingCopy::*;
