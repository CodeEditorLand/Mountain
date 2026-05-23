//! Output-channel domain handlers for `CocoonService`. Five entry points,
//! each forwarding a `sky://output/<verb>` event to Sky.

pub mod AppendOutput;

pub mod ClearOutput;

pub mod CreateOutputChannel;

pub mod DisposeOutput;

pub mod ShowOutput;
