//! `WindEnvironmentService::GetAppRoot`

use super::Struct;


pub fn Fn(This:&Struct) -> Result<String, String> { std::env::var("APP_ROOT").map_err(|E| e.to_string()) }
