//! The localhost plugin base URL (`http://localhost:<port>`).
//! Set once at startup so any subsystem can construct URLs served by the
//! localhost plugin without accessing the port directly.

static LOCALHOST_URL:std::sync::OnceLock<String> = std::sync::OnceLock::new();

pub fn set_localhost_url(Url:String) { let _ = LOCALHOST_URL.set(Url); }

pub fn get_localhost_url() -> Option<String> { LOCALHOST_URL.get().cloned() }
