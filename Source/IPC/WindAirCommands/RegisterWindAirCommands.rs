//! Wire all 9 `WindAirCommands` Tauri commands into a Tauri
//! `Builder`. The dispatcher itself is kept here so the command
//! list stays a single source of truth. Each command is
//! referenced by its full path so the `tauri::command` accessor
//! macro resolves in its own module.

pub fn Fn<R:tauri::Runtime>(builder:tauri::Builder<R>) -> tauri::Builder<R> {
	builder.invoke_handler(tauri::generate_handler![
		crate::IPC::WindAirCommands::Fn::Fn,
		crate::IPC::WindAirCommands::Fn::Fn,
		crate::IPC::WindAirCommands::Fn::Fn,
		crate::IPC::WindAirCommands::Fn::Fn,
		crate::IPC::WindAirCommands::Fn::Fn,
		crate::IPC::WindAirCommands::Fn::Fn,
		crate::IPC::WindAirCommands::Fn::Fn,
		crate::IPC::WindAirCommands::Fn::Fn,
		crate::IPC::WindAirCommands::Fn::Fn,
	])
}
