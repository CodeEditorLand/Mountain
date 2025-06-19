//! # FileSystemProvider Implementation
//!
//! Implements the `FileSystemReader` and `FileSystemWriter` traits for the
//! `MountainEnvironment`, providing the concrete logic for all filesystem
//! operations.

use std::path::{Path, PathBuf};

use Common::{
	Error::CommonError::CommonError,
	FileSystem::{
		DTO::{FileSystemStatDTO, FileTypeDTO},
		FileSystemReader,
		FileSystemWriter,
	},
};
use async_trait::async_trait;
use tokio::fs;

use super::{MountainEnvironment::MountainEnvironment, Utility};

#[async_trait]
impl FileSystemReader for MountainEnvironment {
	/// Reads the entire contents of a file into a byte vector after verifying
	/// access rights.
	async fn ReadFile(&self, Path:&PathBuf) -> Result<Vec<u8>, CommonError> {
		Utility::IsPathAllowedForAccess(&self.ApplicationState, Path)?;
		fs::read(Path)
			.await
			.map_err(|e| CommonError::FromStandardIOError(e, Path.clone(), "ReadFile"))
	}

	/// Retrieves metadata for a file or directory after verifying access
	/// rights.
	async fn StatFile(&self, Path:&PathBuf) -> Result<FileSystemStatDTO, CommonError> {
		Utility::IsPathAllowedForAccess(&self.ApplicationState, Path)?;
		let Metadata = fs::metadata(Path)
			.await
			.map_err(|e| CommonError::FromStandardIOError(e, Path.clone(), "StatFile"))?;

		let mut FileType = 0_u8;
		if Metadata.is_file() {
			FileType |= FileTypeDTO::File as u8;
		}
		if Metadata.is_dir() {
			FileType |= FileTypeDTO::Directory as u8;
		}
		if Metadata.file_type().is_symlink() {
			FileType |= FileTypeDTO::SymbolicLink as u8;
		}

		let GetMilliTimestamp = |SystemTimeResult:Result<std::time::SystemTime, _>| -> u64 {
			SystemTimeResult
				.ok()
				.and_then(|time| time.duration_since(std::time::SystemTime::UNIX_EPOCH).ok())
				.map_or(0, |duration| duration.as_millis() as u64)
		};

		Ok(FileSystemStatDTO {
			FileType,
			CreationTime:GetMilliTimestamp(Metadata.created()),
			ModificationTime:GetMilliTimestamp(Metadata.modified()),
			Size:Metadata.len(),
			Permissions:None, // Permissions are not yet implemented.
		})
	}

	/// Reads the contents of a directory after verifying access rights.
	async fn ReadDirectory(&self, Path:&PathBuf) -> Result<Vec<(String, FileTypeDTO)>, CommonError> {
		Utility::IsPathAllowedForAccess(&self.ApplicationState, Path)?;
		let mut Entries = Vec::new();
		let mut ReadDirectory = fs::read_dir(Path)
			.await
			.map_err(|e| CommonError::FromStandardIOError(e, Path.clone(), "ReadDirectory"))?;

		while let Some(EntryResult) = ReadDirectory
			.next_entry()
			.await
			.map_err(|e| CommonError::FromStandardIOError(e, Path.clone(), "ReadDirectory.NextEntry"))?
		{
			let FileName = EntryResult.file_name().to_string_lossy().into_owned();
			let FileType = match EntryResult.file_type().await {
				Ok(ft) => {
					if ft.is_dir() {
						FileTypeDTO::Directory
					} else if ft.is_file() {
						FileTypeDTO::File
					} else {
						FileTypeDTO::Unknown
					}
				},
				Err(_) => FileTypeDTO::Unknown,
			};
			Entries.push((FileName, FileType));
		}
		Ok(Entries)
	}
}

#[async_trait]
impl FileSystemWriter for MountainEnvironment {
	/// Writes content to a file after verifying access rights and options.
	async fn WriteFile(&self, Path:&PathBuf, Content:Vec<u8>, Create:bool, Overwrite:bool) -> Result<(), CommonError> {
		Utility::IsPathAllowedForAccess(&self.ApplicationState, Path)?;
		let PathExists = fs::try_exists(Path).await.unwrap_or(false);

		if PathExists && !Overwrite {
			return Err(CommonError::FileSystemFileExists(Path.clone()));
		}
		if !PathExists && !Create {
			return Err(CommonError::FileSystemNotFound(Path.clone()));
		}

		if let Some(ParentDirectory) = Path.parent() {
			if !fs::try_exists(ParentDirectory).await.unwrap_or(false) {
				fs::create_dir_all(ParentDirectory).await.map_err(|e| {
					CommonError::FromStandardIOError(e, ParentDirectory.to_path_buf(), "WriteFile.CreateParent")
				})?;
			}
		}

		fs::write(Path, &Content)
			.await
			.map_err(|e| CommonError::FromStandardIOError(e, Path.clone(), "WriteFile"))
	}

	/// Creates a directory after verifying access rights.
	async fn CreateDirectory(&self, Path:&PathBuf, Recursive:bool) -> Result<(), CommonError> {
		Utility::IsPathAllowedForAccess(&self.ApplicationState, Path)?;
		let Operation = if Recursive {
			fs::create_dir_all(Path).await
		} else {
			fs::create_dir(Path).await
		};
		Operation.map_err(|e| CommonError::FromStandardIOError(e, Path.clone(), "CreateDirectory"))
	}

	/// Deletes a file or directory after verifying access rights.
	async fn Delete(&self, Path:&PathBuf, Recursive:bool, _UseTrash:bool) -> Result<(), CommonError> {
		Utility::IsPathAllowedForAccess(&self.ApplicationState, Path)?;
		// A full implementation would use the `trash` crate if `UseTrash` is true.
		match fs::metadata(Path).await {
			Ok(Metadata) => {
				let Operation = if Metadata.is_dir() {
					if Recursive {
						fs::remove_dir_all(Path).await
					} else {
						fs::remove_dir(Path).await
					}
				} else {
					fs::remove_file(Path).await
				};
				Operation.map_err(|e| CommonError::FromStandardIOError(e, Path.clone(), "Delete"))
			},
			Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(()), // Idempotent success
			Err(e) => Err(CommonError::FromStandardIOError(e, Path.clone(), "Delete.Stat")),
		}
	}

	/// Renames (moves) a file or directory after verifying access rights.
	async fn Rename(&self, Source:&PathBuf, Target:&PathBuf, Overwrite:bool) -> Result<(), CommonError> {
		Utility::IsPathAllowedForAccess(&self.ApplicationState, Source)?;
		Utility::IsPathAllowedForAccess(&self.ApplicationState, Target)?;
		if !Overwrite && fs::try_exists(Target).await.unwrap_or(false) {
			return Err(CommonError::FileSystemFileExists(Target.clone()));
		}
		fs::rename(Source, Target)
			.await
			.map_err(|e| CommonError::FromStandardIOError(e, Source.clone(), "Rename"))
	}

	/// Copies a file after verifying access rights.
	async fn Copy(&self, Source:&PathBuf, Target:&PathBuf, Overwrite:bool) -> Result<(), CommonError> {
		Utility::IsPathAllowedForAccess(&self.ApplicationState, Source)?;
		Utility::IsPathAllowedForAccess(&self.ApplicationState, Target)?;
		let SourceMetadata = self.StatFile(Source).await?;
		if (SourceMetadata.FileType & FileTypeDTO::Directory as u8) != 0 {
			return Err(CommonError::NotImplemented { FeatureName:"Recursive directory copy".to_string() });
		}
		if !Overwrite && fs::try_exists(Target).await.unwrap_or(false) {
			return Err(CommonError::FileSystemFileExists(Target.clone()));
		}
		fs::copy(Source, Target)
			.await
			.map(|_| ())
			.map_err(|e| CommonError::FromStandardIOError(e, Source.clone(), "Copy"))
	}

	/// Creates a new, empty file after verifying access rights.
	async fn CreateFile(&self, Path:&PathBuf) -> Result<(), CommonError> {
		// Use WriteFile with an empty Vec, ensuring creation without overwrite.
		self.WriteFile(Path, vec![], true, false).await
	}
}
