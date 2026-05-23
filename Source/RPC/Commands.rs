//! Command-registration RPC service. Three sub-files: `CommandService` (the
//! impl handle), `CommandValidation` (input checks), `Command` (the DTO).

pub mod Command;

pub mod CommandService;

pub mod CommandValidation;
