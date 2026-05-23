//! Search-results envelope returned by `SearchFiles`.

use serde::{Deserialize, Serialize};

use crate::IPC::WindAirCommands::FileResultDTO;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Struct {
	pub results:Vec<FileResultDTO::Struct>,

	pub total_results:u32,
}
