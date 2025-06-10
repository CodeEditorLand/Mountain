

/**
 * @module Mountain Library
 * @description This file conceptually represents the library root for the Mountain
 * application, declaring all of its major internal components.
 */

#![allow(non_snake_case, non_camel_case_types)]

pub mod app_state;
pub mod environment;
pub mod handlers;
pub mod runtime;
pub mod scheduler; // Mountain owns the scheduler instance
pub mod track;
pub mod vine;
