//! Health classification for `ConnectionHandle::Struct` -
//! `Healthy` (default), `Unhealthy` (failed health check),
//! `Degraded` (degraded but still usable).

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Enum {
	Healthy,

	Unhealthy,

	Degraded,
}
