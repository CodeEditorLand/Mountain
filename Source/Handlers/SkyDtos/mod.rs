
// This module defines and exports Data Transfer Objects (DTOs) used for
// communication with the Sky (frontend) layer, particularly for initial
// configuration.

#![allow(non_snake_case, non_camel_case_types)]

mod SkyDtos; // Contains the DTO struct definitions

pub use self::SkyDtos::*; // Re-export all public items from SkyDtos.rs
