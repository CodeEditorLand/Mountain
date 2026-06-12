//! Debug-domain handlers for `CocoonService`.
//! `RegisterDebugAdapter::Fn`, `StartDebugging::Fn`, `StopDebugging::Fn`.
/// RegisterDebugAdapter handler: registers a debug adapter with the
/// environment.
pub mod RegisterDebugAdapter;

/// StartDebugging handler: starts a debugging session.
pub mod StartDebugging;

/// StopDebugging handler: stops an active debugging session.
pub mod StopDebugging;
