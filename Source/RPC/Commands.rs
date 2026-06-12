//! Command-registration RPC service. Three sub-files: `CommandService` (the
//! impl handle), `CommandValidation` (input checks), `Command` (the DTO).
/// Command DTO: describes a registered command with id, title, and optional
/// description.
pub mod Command;

/// Command service: registers, finds, and executes commands on behalf of the
/// extension host.
pub mod CommandService;

/// Command validation: checks command input before dispatch.
pub mod CommandValidation;
