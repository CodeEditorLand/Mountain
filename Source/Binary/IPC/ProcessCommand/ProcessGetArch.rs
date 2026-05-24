//! Tauri command - return the CPU architecture string
//! (`x86_64` / `aarch64` / …) from `std::env::consts::ARCH`.

#[tauri::command]
pub async fn Fn() -> Result<String, String> { Ok(std::env::consts::ARCH.to_string()) }
