//! `WindEnvironmentService::GetUserDataPath`

use super::Struct;


pub fn Fn(This:&Struct) -> Result<String, String> {
		std::env::var("USER_DATA_PATH").map_err(|E| e.to_string())
	}
