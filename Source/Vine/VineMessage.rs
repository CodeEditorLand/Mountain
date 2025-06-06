// File: Vine/VineMessage.rs
// Defines the legacy Vine message structures for stdio-based IPC.
// This is likely deprecated in favor of gRPC messages defined in .proto files.

#![allow(non_snake_case, non_camel_case_types)]

use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq)]
#[serde(rename_all = "PascalCase")]
pub enum VineMessageType {
	Request = 1,
	Response = 3,
	Error = 4,
	Cancel = 5,
	Notification = 6,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "PascalCase")]
pub struct VineMessage {
	#[serde(alias = "msg_type")]
	pub MessageType:VineMessageType,
	#[serde(skip_serializing_if = "Option::is_none")]
	pub Id:Option<u64>,
	#[serde(skip_serializing_if = "Option::is_none")]
	pub Method:Option<String>,
	#[serde(skip_serializing_if = "Option::is_none")]
	#[serde(alias = "params")]
	pub Parameters:Option<Value>,
	#[serde(skip_serializing_if = "Option::is_none")]
	pub Error:Option<Value>,
}
