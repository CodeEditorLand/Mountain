// ---------------------------------------------------------------------------------------------
// Mountain Environment - Filesystem Provider (environment/fs_provider.rs)
// --------------------------------------------------------------------------------------------
// This module implements the `FsReader` and `FsWriter` traits for
// `MountainEnvironment`, providing the core filesystem operations used by
// effects. It leverages tokio::fs for asynchronous file I/O and performs
// security checks to restrict access to authorized paths only.
// --------------------------------------------------------------------------------------------

use std::{path::PathBuf, sync::Arc};

use Land_Common::{
	environment::Requires, // Core Environment trait and Requires helper
	errors::CommonError,   // Error enum
	fs_effects::{FileSystemStat, FileType as CommonFileType, FsReader, FsWriter},
};
use async_trait::async_trait;
use tokio::fs;

use crate::environment::{
	MountainEnvironment,
	utils,
	utils::{is_path_allowed_for_filesystem_access, map_io_error_to_common_error},
}; // Access to struct, helpers // Security check
// use crate::app_state::AppState; // Not needed for this isolated
// implementation

// --- FsReader Implementation ---
#[async_trait]
impl FsReader for MountainEnvironment {
	async fn read_file(&self, path:&PathBuf) -> Result<Vec<u8>, CommonError> {
		// Security check (ensure path is within allowed workspace or data folders).
		utils::is_path_allowed_for_filesystem_access(self, path).await?;

		trace!("[Env FsReader] Reading file: {}", path.display());

		// Use tokio::fs for asynchronous file read.
		fs::read(path)
			.await
			.map_err(|io_err| utils::map_io_error_to_common_error(io_err, path.clone(), "read"))
	}

	async fn stat_file(&self, path:&PathBuf) -> Result<FileSystemStat, CommonError> {
		// Security check
		utils::is_path_allowed_for_filesystem_access(self, path).await?;

		trace!("[Env FsReader] Stating file/directory: {}", path.display());

		match tokio::fs::metadata(path).await {
			Ok(metadata) => {
				// Set type flags and timestamp
				let mut file_type_flags = 0_u8;
				if metadata.is_file() {
					file_type_flags |= CommonFileType::File as u8;
				}
				if metadata.is_dir() {
					file_type_flags |= CommonFileType::Directory as u8;
				}
				if metadata.is_symlink() {
					file_type_flags |= CommonFileType::SymbolicLink as u8;
				}

				let get_milli_timestamp_from_system_time = |sys_time_res:Result<std::time::SystemTime, _>| -> u64 {
					sys_time_res
						.ok()
						.and_then(|time| time.duration_since(std::time::SystemTime::UNIX_EPOCH).ok())
						.map_or(0, |duration| duration.as_millis() as u64)
				};

				Ok(FileSystemStat {
					file_type:file_type_flags, // Combination of FileType flags
					ctime:get_milli_timestamp_from_system_time(metadata.created()),
					mtime:get_milli_timestamp_from_system_time(metadata.modified()),
					size:metadata.len(),
					permissions:None,
				})
			},
			Err(io_err) => Err(utils::map_io_error_to_common_error(io_err, path.clone(), "stat")),
		}
	}

	async fn read_directory(&self, path:&PathBuf) -> Result<Vec<(String, CommonFileType)>, CommonError> {
		// Security check
		utils::is_path_allowed_for_filesystem_access(path).await?;

		debug!("[Env FsReader] Reading directory contents: {}", path.display());

		let mut entries_vec:Vec<(String, CommonFileType)> = Vec::new();

		let mut dir_entries_stream = fs::read_dir(path)
			.await
			.map_err(|io_err| utils::map_io_error_to_common_error(io_err, path.clone(), "readdir"))?;

		while let Some(dir_entry_res) = dir_entries_stream
			.next_entry()
			.await
			.map_err(|io_err| utils::map_io_error_to_common_error(io_err, path.clone(), "readdir_next_entry"))?
		{
			let file_name_osstr = dir_entry_res.file_name();
			let file_name_str = file_name_osstr.to_string_lossy().into_owned();

			match dir_entry_res.file_type().await {
				Ok(file_type_tokio) => {
					let common_file_type = if file_type_tokio.is_dir() {
						CommonFileType::Directory
					} else if file_type_tokio.is_file() {
						CommonFileType::File
					} else if file_type_tokio.is_symlink() {
						CommonFileType::SymbolicLink
					} else {
						CommonFileType::Unknown
					};
					entries_vec.push((file_name_str, common_file_type));
				},
				Err(e_ftype) => {
					warn!(
						"[Env FsReader] Failed to get file type for entry '{}' in directory '{}': {}. Marking as \
						 Unknown.",
						file_name_str,
						path.display(),
						e_ftype
					);
					entries_vec.push((file_name_str, CommonFileType::Unknown));
				},
			}
		}

		Ok(entries_vec)
	}
}

// --- FsWriter Implementation ---
#[async_trait]
impl FsWriter for MountainEnvironment {
	async fn write_file(
		&self,
		path:&PathBuf,
		content_bytes:Vec<u8>,
		create_if_not_exists:bool,
		overwrite_if_exists:bool,
	) -> Result<(), CommonError> {
		// Security check
		utils::is_path_allowed_for_filesystem_access(path).await?;

		info!(
			"[Env FsWriter] Writing file: path='{}', len={}, create={}, overwrite={}",
			path.display(),
			content_bytes.len(),
			create_if_not_exists,
			overwrite_if_exists
		);

		let path_exists = fs::try_exists(path).await.unwrap_or(false);

		if path_exists && !overwrite_if_exists {
			return Err(CommonError::FsFileExists(path.clone()));
		}

		if !path_exists && !create_if_not_exists {
			return Err(CommonError::FsNotFound(path.clone()));
		}

		if let Some(parent_dir_path) = path.parent() {
			if !fs::try_exists(parent_dir_path).await.unwrap_or(false) {
				if create_if_not_exists {
					fs::create_dir_all(parent_dir_path).await.map_err(|io_err| {
						map_io_error_to_common_error(io_err, parent_dir_path.to_path_buf(), "mkdir_parent_for_write")
					})?;
				} else {
					return Err(CommonError::FsNotFound(parent_dir_path.to_path_buf()));
				}
			}
		}

		fs::write(path, &content_bytes)
			.await
			.map_err(|io_err| map_io_error_to_common_error(io_err, path.clone(), "write"))?;

		// TODO: Emit filesystem_changed event.
		Ok(())
	}

	async fn create_directory(&self, path:&PathBuf, recursive_create:bool) -> Result<(), CommonError> {
		// Security check
		utils::is_path_allowed_for_filesystem_access(path).await?;

		info!(
			"[Env FsWriter] Creating directory: path='{}', recursive={}",
			path.display(),
			recursive_create
		);

		if recursive_create {
			fs::create_dir_all(path)
				.await
				.map_err(|io_err| map_io_error_to_common_error(io_err, path.clone(), "mkdir_all"))?;
		} else {
			fs::create_dir(path)
				.await
				.map_err(|io_err| map_io_error_to_common_error(io_err, path.clone(), "mkdir"))?;
		}

		// TODO: Emit filesystem_changed event.
		Ok(())
	}

	async fn delete(&self, path:&PathBuf, recursive_delete:bool, use_os_trash:bool) -> Result<(), CommonError> {
		// Security check
		self.is_path_allowed_for_filesystem_access(path).await?;

		info!(
			"[Env FsWriter] Deleting: path='{}', recursive={}, useTrash={}",
			path.display(),
			recursive_delete,
			use_os_trash
		);

		if use_os_trash {
			warn!(
				"[Env FsWriter] 'useTrash=true' option for delete is requested but not yet implemented. Performing \
				 permanent delete."
			);
		}

		match fs::metadata(path).await {
			Ok(metadata) => {
				let delete_operation_result = if metadata.is_dir() {
					if recursive_delete {
						fs::remove_dir_all(path).await
					} else {
						fs::remove_dir(path).await
					}
				} else {
					fs::remove_file(path).await
				};

				delete_operation_result
					.map_err(|io_err| map_io_error_to_common_error(io_err, path.clone(), "delete"))?;

				// TODO: Emit filesystem_changed event.
				Ok(())
			},
			Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
				// Deleting a non-existent path is considered success (idempotent).
				debug!(
					"[Env FsWriter] Path '{}' not found for deletion. Operation considered successful (idempotent).",
					path.display()
				);
				Ok(())
			},
			Err(io_err) => Err(map_io_error_to_common_error(io_err, path.clone(), "delete_stat_check")),
		}
	}

	async fn rename(
		&self,
		source_path:&PathBuf,
		target_path:&PathBuf,
		overwrite_if_target_exists:bool,
	) -> Result<(), CommonError> {
		self.is_path_allowed_for_filesystem_access(source_path).await?;
		self.is_path_allowed_for_filesystem_access(target_path).await?;

		info!(
			"[Env FsWriter] Renaming: from='{}', to='{}', overwrite={}",
			source_path.display(),
			target_path.display(),
			overwrite_if_target_exists
		);

		if !fs::try_exists(source_path).await.unwrap_or(false) {
			return Err(CommonError::FsNotFound(source_path.clone()));
		}

		if !overwrite_if_target_exists && fs::try_exists(target_path).await.unwrap_or(false) {
			return Err(CommonError::FsFileExists(target_path.clone()));
		}

		// If overwriting, and target exists, delete target first.
		// `fs::rename` behavior with existing target can be platform-dependent.
		if overwrite_if_target_exists && fs::try_exists(target_path).await.unwrap_or(false) {
			debug!(
				"[Env FsWriter] Rename: Overwriting target by first deleting '{}'",
				target_path.display()
			);
			let target_metadata = fs::metadata(target_path).await.map_err(|io_err| {
				map_io_error_to_common_error(io_err, target_path.clone(), "rename_target_stat_for_overwrite_delete")
			})?;
			self.delete(target_path, target_metadata.is_dir(), false).await?;
		}

		// Ensure target's parent directory exists.
		if let Some(target_parent_dir) = target_path.parent() {
			if !fs::try_exists(target_parent_dir).await.unwrap_or(false) {
				fs::create_dir_all(target_parent_dir).await.map_err(|io_err| {
					map_io_error_to_common_error(io_err, target_parent_dir.to_path_buf(), "mkdir_parent_for_rename")
				})?;
			}
		}

		fs::rename(source_path, target_path)
			.await
			.map_err(|io_err| map_io_error_to_common_error(io_err, source_path.clone(), "rename"))?;

		// TODO: Emit filesystem_changed events (one delete for source, one create for
		// target, or a specific rename event).
		Ok(())
	}

	async fn copy(
		&self,
		source_path:&PathBuf,
		target_path:&PathBuf,
		overwrite_if_target_exists:bool,
	) -> Result<(), CommonError> {
		self.is_path_allowed_for_filesystem_access(source_path).await?;
		self.is_path_allowed_for_filesystem_access(target_path).await?;

		info!(
			"[Env FsWriter] Copying: from='{}', to='{}', overwrite={}",
			source_path.display(),
			target_path.display(),
			overwrite_if_target_exists
		);

		if !fs::try_exists(source_path).await.unwrap_or(false) {
			return Err(CommonError::FsNotFound(source_path.clone()));
		}

		if !overwrite_if_target_exists && fs::try_exists(target_path).await.unwrap_or(false) {
			return Err(CommonError::FsFileExists(target_path.clone()));
		}

		let source_metadata = fs::metadata(source_path)
			.await
			.map_err(|io_err| map_io_error_to_common_error(io_err, source_path.clone(), "copy_source_stat"))?;

		// TODO: Implement recursive directory copy if `source_metadata.is_dir()`.
		// `tokio::fs::copy` only copies files.
		if source_metadata.is_dir() {
			error!(
				"[Env FsWriter] Recursive directory copy from '{}' is not yet implemented.",
				source_path.display()
			);
			return Err(CommonError::NotImplemented(
				"Recursive directory copy for vscode.workspace.fs.copy".to_string(),
			));
		}

		if overwrite_if_target_exists && fs::try_exists(target_path).await.unwrap_or(false) {
			debug!(
				"[Env FsWriter] Copy: Overwriting target by first deleting '{}'",
				target_path.display()
			);
			self.delete(target_path, false, false).await?;
		}

		// Ensure target's parent directory exists.
		if let Some(target_parent_dir) = target_path.parent() {
			if !fs::try_exists(target_parent_dir).await.unwrap_or(false) {
				fs::create_dir_all(target_parent_dir).await.map_err(|io_err| {
					map_io_error_to_common_error(io_err, target_parent_dir.to_path_buf(), "mkdir_parent_for_copy")
				})?;
			}
		}

		fs::copy(source_path, target_path)
            .await
             // Discard bytes copied, return unit on success
            .map(|_bytes_copied| ())

            .map_err(|io_err| map_io_error_to_common_error(io_err, source_path.clone(), "copy"))?;

		// TODO: Emit filesystem_changed event for target creation.
		Ok(())
	}
}

impl Requires<Arc<dyn FsWriter + Send + Sync>> for MountainEnvironment {
	fn require(&self) -> Arc<dyn FsWriter + Send + Sync> { Arc::new(self.clone()) }
}
