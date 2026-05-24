//! Central request dispatcher that routes commands from the Sky frontend and
//! Cocoon sidecar into strongly-typed ActionEffects executed by the runtime.

// --- Sub-modules ---

/// Frontend command dispatch handling.
pub mod FrontendCommand;

/// Sidecar RPC request dispatch handling.
pub mod SideCarRequest;

/// UI request-response result handling.
pub mod UIRequest;

/// Webview message forwarding.
pub mod Webview;

/// Effect creation and routing.
pub mod Effect;

// No `pub use` re-exports - callers spell the full path
// (`Track::UIRequest::Fn::Fn`, etc.). The
// double-segment shape is required by `tauri::generate_handler!` so the
// macro can find the `__cmd__<Name>` companion in the same file.
