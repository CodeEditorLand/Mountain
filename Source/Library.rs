// @module Mountain Library
// @description This file conceptually represents the library root for the
// Mountain application, declaring all of its major internal components.
#![allow(non_snake_case, non_camel_case_types)]

pub mod ApplicationState;
pub mod Environment;
pub mod Handler;
pub mod RunTime;
// TODO: FIND MISSING SCHEDULER
// pub mod Scheduler; // Mountain owns the scheduler instance
pub mod Binary;
pub mod Track;
pub mod Vine;

#[allow(dead_code)]
#[cfg_attr(mobile, tauri::mobile_entry_point)]
fn main() { Binary::Fn::Fn(); }
