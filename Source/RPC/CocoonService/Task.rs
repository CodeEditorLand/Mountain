#![allow(non_snake_case)]

//! Task-domain handlers for `CocoonService`.
//! `RegisterTaskProvider::Fn`, `ExecuteTask::Fn`, `TerminateTask::Fn`.

pub mod ExecuteTask;
pub mod RegisterTaskProvider;
pub mod TerminateTask;
