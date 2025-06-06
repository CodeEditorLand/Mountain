
// This module is responsible for the "Mist" WebSocket server functionality,
// if it's enabled. It handles WebSocket connections, message parsing,
// and communication between the WebSocket clients and the Mountain backend.

mod Mist;
mod MistServerError; // Defines errors specific to the Mist server // Contains the main WebSocket server logic

pub use self::Mist::*; // Re-export main server functions
pub use self::MistServerError::MistServerError;
