// File: Handlers/Ui/Mod.rs
// This module defines and exports handlers for basic UI interactions,
// specifically for showing simple native message dialogs.

#![allow(non_snake_case, non_camel_case_types)]

mod Ui; // Contains the logic for the basic message dialog handler

pub use self::Ui::*; // Re-export all public functions from Ui.rs
