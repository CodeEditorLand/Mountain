// File: Environment/Ui/Uirequestpayload.rs
// Defines a generic payload structure for sending UI requests to the frontend.

#![allow(non_snake_case, non_camel_case_types)]

use serde::Serialize;

/// A generic struct to wrap UI requests sent to the Sky frontend.
/// This ensures all UI requests have a consistent shape, including a unique
/// identifier for tracking the response.
#[derive(Serialize, Clone, Debug)]
pub struct UiRequestPayload<PayloadType:Serialize + Clone> {
	// A unique identifier for this specific UI request.
	#[serde(alias = "requestId")]
	pub RequestIdentifier:String,
	// The specific payload for the UI request (e.g., dialog options, quick pick items).
	pub Payload:PayloadType,
}
