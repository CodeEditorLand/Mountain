#![allow(non_snake_case)]

//! Command-domain handlers for `CocoonService`.
//! `RegisterCommand::Fn`, `ExecuteContributedCommand::Fn`,
//! `UnregisterCommand::Fn`.

pub mod ExecuteContributedCommand;

pub mod RegisterCommand;

pub mod UnregisterCommand;
