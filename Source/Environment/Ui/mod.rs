
// Declares and exports modules related to UI interaction within the
// environment.

#![allow(non_snake_case, non_camel_case_types)]

// This module defines the generic payload structure for UI requests.
mod Uirequestpayload;

// Re-export the primary struct for use in other parts of the environment.
pub use self::Uirequestpayload::UiRequestPayload;
