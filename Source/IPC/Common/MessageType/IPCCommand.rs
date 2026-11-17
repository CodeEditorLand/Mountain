//! IPC command request: command name + positional `Args` + named
//! `Params` map + a priority. Built through `new` and the `WithArg` /
//! `WithParam` / `WithPriority` builder shims.

use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use crate::IPC::Common::MessageType::MessagePriority;

/// Kind of an IPC command envelope.
///
/// `Request` is the pre-existing shape and stays first, so downstream
/// matches keep their ordering and the wire format of a normal request is
/// unchanged (`Kind` is skipped by serde in this form). `Cancel` is the
/// caller-side cancellation variant: it is keyed by the SAME
/// `CorrelationId` the request already carries (copied from
/// `IPCMessage::Struct::CorrelationId`), not by a second identifier scheme.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum Enum {
	Request,

	Cancel { CorrelationId:String },
}

impl Default for Enum {
	fn default() -> Self { Self::Request }
}

impl Enum {
	/// True only for the implicit normal form; used by serde to keep the
	/// `Kind` key off the wire for normal requests.
	pub fn IsRequest(&self) -> bool { matches!(self, Self::Request) }

	/// The existing correlation id carried by a `Cancel` variant.
	pub fn CorrelationId(&self) -> Option<&str> {
		match self {
			Self::Request => None,

			Self::Cancel { CorrelationId } => Some(CorrelationId.as_str()),
		}
	}
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Struct {
	pub Command:String,

	pub Args:Vec<String>,

	pub Params:HashMap<String, serde_json::Value>,

	pub Priority:MessagePriority::Enum,

	/// Cancellation variant. Absent (default `Request`) on the wire for a
	/// normal request; `Cancel { CorrelationId }` targets an in-flight
	/// request by its existing correlation id.
	#[serde(default, skip_serializing_if = "Enum::IsRequest")]
	pub Kind:Enum,
}

impl Struct {
	pub fn new(Command:impl Into<String>) -> Self {
		Self {
			Command:Command.into(),

			Args:Vec::new(),

			Params:HashMap::new(),

			Priority:MessagePriority::Enum::Normal,

			Kind:Enum::Request,
		}
	}

	pub fn WithArg(mut self, Arg:impl Into<String>) -> Self {
		self.Args.push(Arg.into());

		self
	}

	pub fn WithParam(mut self, Key:impl Into<String>, Value:serde_json::Value) -> Self {
		self.Params.insert(Key.into(), Value);

		self
	}

	pub fn WithPriority(mut self, Priority:MessagePriority::Enum) -> Self {
		self.Priority = Priority;

		self
	}

	/// Mark this command as the cancellation of `CorrelationId`. The id is
	/// the one already assigned to the in-flight request - no new id is
	/// minted. Callers that lack an id simply leave `Kind` as `Request`.
	pub fn WithCancel(mut self, CorrelationId:impl Into<String>) -> Self {
		self.Kind = Enum::Cancel { CorrelationId:CorrelationId.into() };

		self
	}

	/// True when this command is the `Cancel` variant.
	pub fn IsCancel(&self) -> bool { self.Kind.CorrelationId().is_some() }
}
