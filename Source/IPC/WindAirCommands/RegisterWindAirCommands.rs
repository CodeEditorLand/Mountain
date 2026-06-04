//! Wire all 9 `WindAirCommands` Tauri commands into a Tauri
//! `Builder`. The dispatcher itself is kept here so the command
//! list stays a single source of truth. Each command is
//! referenced by its full path so the `tauri::command` accessor
//! macro resolves in its own module.

pub fn Fn<R:tauri::Runtime>(builder:tauri::Builder<R>) -> tauri::Builder<R> {

	builder.invoke_handler(tauri::generate_handler![
		crate::IPC::WindAirCommands::CheckForUpdates::CheckForUpdates,

		crate::IPC::WindAirCommands::DownloadUpdate::DownloadUpdate,

		crate::IPC::WindAirCommands::ApplyUpdate::ApplyUpdate,

		crate::IPC::WindAirCommands::DownloadFile::DownloadFile,

		crate::IPC::WindAirCommands::AuthenticateUser::AuthenticateUser,

		crate::IPC::WindAirCommands::IndexFiles::IndexFiles,

		crate::IPC::WindAirCommands::SearchFiles::SearchFiles,

		crate::IPC::WindAirCommands::GetAirStatus::GetAirStatus,

		crate::IPC::WindAirCommands::GetAirMetrics::GetAirMetrics,
	])
}
