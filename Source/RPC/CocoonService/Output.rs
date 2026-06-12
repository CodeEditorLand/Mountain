//! Output-channel domain handlers for `CocoonService`. Five entry points,
//! each forwarding a `sky://output/<verb>` event to Sky.
/// AppendOutput handler: appends text to an output channel.
pub mod AppendOutput;

/// ClearOutput handler: clears all text from an output channel.
pub mod ClearOutput;

/// CreateOutputChannel handler: creates a new output channel.
pub mod CreateOutputChannel;

/// DisposeOutput handler: disposes an output channel.
pub mod DisposeOutput;

/// ShowOutput handler: reveals an output channel in the UI.
pub mod ShowOutput;
