#![feature(rwlock_data_ptr)]

//! Mountain binary entry point
//!
//! This file serves as the main entry point for the Mountain application.
//! It delegates to the library's Binary module.

fn main() { Mountain::Binary::Main(); }
