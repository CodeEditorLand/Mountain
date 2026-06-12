//! Command-domain handlers for `CocoonService`.
//! `ExecuteContributedCommand::Fn`, `RegisterCommand::Fn`,
//! `UnregisterCommand::Fn`.
/// ExecuteContributedCommand handler: dispatches a command contributed by an
/// extension.
pub mod ExecuteContributedCommand;

/// RegisterCommand handler: registers a command contributed by an extension.
pub mod RegisterCommand;

/// UnregisterCommand handler: removes a previously registered command.
pub mod UnregisterCommand;
