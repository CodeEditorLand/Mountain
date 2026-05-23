//! Network endpoint metadata: protocol, host, port, optional Unix-domain
//! socket path. `NewUnix` is the convenience constructor for the
//! UDS-only path; everything else uses the four-arg `new`.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Struct {
	pub Protocol:String,

	pub Address:String,

	pub Port:u16,

	pub Path:Option<String>,
}

impl Struct {
	pub fn new(Protocol:impl Into<String>, Address:impl Into<String>, Port:u16) -> Self {
		Self { Protocol:Protocol.into(), Address:Address.into(), Port, Path:None }
	}

	pub fn NewUnix(Path:impl Into<String>) -> Self {
		Self {
			Protocol:"unix".to_string(),

			Address:String::new(),

			Port:0,

			Path:Some(Path.into()),
		}
	}
}
