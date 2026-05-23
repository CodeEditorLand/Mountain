
//! Wind-shaped environment-variable accessor. Reads
//! `APP_ROOT` / `USER_DATA_PATH` from the process env so Wind's
//! desktop bootstrap can stay agnostic to the Tauri side.

pub struct Struct {}

impl Struct {
	pub fn new() -> Self { Self {} }

	pub async fn get_app_root(&self) -> Result<String, String> { std::env::var("APP_ROOT").map_err(|e| e.to_string()) }

	pub async fn get_user_data_path(&self) -> Result<String, String> {
		std::env::var("USER_DATA_PATH").map_err(|e| e.to_string())
	}
}
