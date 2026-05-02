#![allow(non_snake_case)]

//! Bridge between the declarative `ActionEffect` system and the Echo
//! work-stealing scheduler. Three entry points: bare `Run` (trait method),
//! `RunWithTimeout`, and `RunWithRetry`.

pub mod Run;
pub mod RunWithRetry;
pub mod RunWithTimeout;
