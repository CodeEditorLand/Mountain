pub mod New;
pub mod RecordReceived;
pub mod RecordSent;
pub mod MessagesPerSecondReceived;
pub mod MessagesPerSecondSent;
pub mod BytesPerSecondReceived;
pub mod BytesPerSecondSent;

use std::time::Instant;

#[derive(Debug, Clone)]
pub struct Struct {
	pub MessagesReceived:u64,

	pub MessagesSent:u64,

	pub BytesReceived:u64,

	pub BytesSent:u64,

	pub StartTime:Instant,
}
