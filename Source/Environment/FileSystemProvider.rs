//! # FileSystemProvider (Environment)
//!
//! RESPONSIBILITIES:
//! - Implements
//!   [`FileSystemReader`](CommonLibrary::FileSystem::FileSystemReader) and
//!   [`FileSystemWriter`](CommonLibrary::FileSystem::FileSystemWriter) for
//!   [`MountainEnvironment`]
//! - Provides secure, validated filesystem access with workspace trust
//!   enforcement
//! - Handles file operations: read, write, stat, delete, rename, copy,
//!   directory traversal
//! - Detects and handles symbolic links properly
//! - Enforces path validation to prevent directory traversal attacks
//!
//! SECURITY MODEL:
//! - Sandboxed filesystem access limited to registered workspace folders
//! - All operations call [`Utility::IsPathAllowedForAccess`](crate::Utility)
//!   first
//! - Requires workspace trust to be enabled for any file access
//! - Path normalization prevents `../` attacks
//! - Symbolic link detection avoids following untrusted links outside
//!   workspaces
//!
//! ERROR HANDLING:
//! - Uses [`CommonError`](CommonLibrary::Error::CommonError) for all operations
//! - File operation errors are mapped via `CommonError::FromStandardIOError`
//! - Validates paths are within workspace boundaries (IsPathAllowedForAccess)
//! - Rejects directory reads when file expected (ReadFile)
//!
//! PERFORMANCE:
//! - Uses async tokio::fs for non-blocking I/O operations
//! - Symbolic link detection uses `symlink_metadata` in addition to `metadata`
//! - TODO: Consider caching file metadata for frequently accessed files
//!
//! VS CODE REFERENCE:
//! - `vs/workbench/services/files/electron-browser/diskFileSystemProvider.ts` -
//!   secure FS access
//! - `vs/platform/files/common/files.ts` - file system interfaces
//! - `vs/base/common/network.ts` - URI and path handling
//!
//! TODO:
//! - Implement filesystem change watching (notify, inotify, FSEvents)
//! - Add path normalization to prevent directory traversal
//! - Implement proper symbolic link resolution with security checks
//! - Add support for file permissions and ownership metadata
//! - Implement atomic file writes using temp file + rename pattern
//! - Add filesystem usage statistics (disk space, file counts)
//! - Implement file attribute querying (hidden, readonly, executable)
//! - Add support for extended file attributes on Unix/macOS
//! - Consider adding filesystem cache for metadata
//! - Implement trash operation using platform trash API (not delete)
//! - Add support for file system encoding detection
//! - Implement case sensitivity handling based on filesystem type
//!
//! MODULE CONTENTS:
//! - [`FileSystemReader`](CommonLibrary::FileSystem::FileSystemReader)
//!   implementation:
//!   - [`ReadFile`](Self::ReadFile) - read file bytes with access validation
//!   - [`StatFile`](Self::StatFile) - file/directory metadata with symlink
//!     detection
//! - [`FileSystemWriter`](CommonLibrary::FileSystem::FileSystemWriter)
//!   implementation:
//!   - (methods to be implemented: WriteFile, DeleteFile, CreateDirectory,
//!     etc.)
//! - Data types:
//!   [`FileSystemStatDTO`](CommonLibrary::FileSystem::DTO::FileSystemStatDTO),
//!   [`FileTypeDTO`](CommonLibrary::FileSystem::DTO::FileTypeDTO)

use std::path::PathBuf;

use CommonLibrary::{
	Error::CommonError::CommonError,
	FileSystem::{
		DTO::{FileSystemStatDTO::FileSystemStatDTO, FileTypeDTO::FileTypeDTO},
		FileSystemReader::FileSystemReader,
		FileSystemWriter::FileSystemWriter,
	},
};
use async_trait::async_trait;
use tokio::fs;

use super::{MountainEnvironment::MountainEnvironment, Utility};

#[async_trait]
impl FileSystemReader for MountainEnvironment {
	/// Reads the entire contents of a file into a byte vector after verifying
	/// access rights. Returns an error if the path points to a directory.
	async fn ReadFile(&self, Path:&PathBuf) -> Result<Vec<u8>, CommonError> {
		Utility::IsPathAllowedForAccess(&self.ApplicationState, Path)?;

		// Validate that the path exists and is a file, not a directory
		let Metadata = fs::metadata(Path)
			.await
			.map_err(|Error| CommonError::FromStandardIOError(Error, Path.clone(), "ReadFile.Stat"))?;

		if Metadata.is_dir() {
			return Err(CommonError::InvalidArgument {
				ArgumentName:"Path".to_string(),
				Reason:format!("Cannot read directory as file: {}", Path.display()),
			});
		}

		fs::read(Path)
			.await
			.map_err(|Error| CommonError::FromStandardIOError(Error, Path.clone(), "ReadFile"))
	}

	/// Retrieves metadata for a file or directory after verifying access
	/// rights. Includes symbolic link detection and timestamp handling.
	async fn StatFile(&self, Path:&PathBuf) -> Result<FileSystemStatDTO, CommonError> {
		Utility::IsPathAllowedForAccess(&self.ApplicationState, Path)?;

		let Metadata = fs::metadata(Path)
			.await
			.map_err(|Error| CommonError::FromStandardIOError(Error, Path.clone(), "StatFile"))?;

		let mut FileType = 0_u8;

		if Metadata.is_file() {
			FileType |= FileTypeDTO::File as u8;
		}

		if Metadata.is_dir() {
			FileType |= FileTypeDTO::Directory as u8;
		}

		// Check for symbolic link separately using symlink_metadata()
		let FileTypeRaw = fs::symlink_metadata(Path)
			.await
			.map_err(|Error| CommonError::FromStandardIOError(Error, Path.clone(), "StatFile.FileType"))?;

		if FileTypeRaw.is_symlink() {
			FileType |= FileTypeDTO::SymbolicLink as u8;
		}

		// Note: Windows typically doesn't support creation_time, handle gracefully
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

			// Capture file permissions by extracting Unix file mode (st_mode) and Windows
			// file attributes. On Unix, extract permission bits (rwx for owner/group/others)
			// and store in FileSystemPermissionsDTO. On Windows, capture attributes
			// (readonly, hidden, system, archive). This enables preserving permissions
			// during file operations and respecting the user's filesystem ACLs. Currently
			// returns None, which defaults to inherited permissions.
			Permissions:None,
		})
	}

	/// Reads the contents of a directory after verifying access rights.
	/// Returns a list of file/directory names along with their types.
	/// Properly handles symbolic links and hidden files.
	async fn ReadDirectory(&self, Path:&PathBuf) -> Result<Vec<(String, FileTypeDTO)>, CommonError> {
		Utility::IsPathAllowedForAccess(&self.ApplicationState, Path)?;

		// Validate that the path exists and is a directory
		let Metadata = fs::metadata(Path)
			.await
			.map_err(|Error| CommonError::FromStandardIOError(Error, Path.clone(), "ReadDirectory.Stat"))?;

		if !Metadata.is_dir() {
			return Err(CommonError::InvalidArgument {
				ArgumentName:"Path".to_string(),
				Reason:format!("Cannot read directory: path is not a directory: {}", Path.display()),
			});
		}

		let mut Entries = Vec::new();

		let mut ReadDirectory = fs::read_dir(Path)
			.await
			.map_err(|Error| CommonError::FromStandardIOError(Error, Path.clone(), "ReadDirectory"))?;

		while let Some(EntryResult) = ReadDirectory
			.next_entry()
			.await
			.map_err(|Error| CommonError::FromStandardIOError(Error, Path.clone(), "ReadDirectory.NextEntry"))?
		{
			let FileName = EntryResult.file_name().to_string_lossy().into_owned();

			// Determine file type including symbolic link detection
			let FileType = match EntryResult.file_type().await {
				Ok(ft) => {
					if ft.is_symlink() {
						FileTypeDTO::SymbolicLink
					} else if ft.is_dir() {
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
	/// Creates parent directories if they don't exist when Create is true.
	async fn WriteFile(&self, Path:&PathBuf, Content:Vec<u8>, Create:bool, Overwrite:bool) -> Result<(), CommonError> {
		Utility::IsPathAllowedForAccess(&self.ApplicationState, Path)?;

		// Validate that Content is not excessively large to prevent memory issues
		if Content.len() > 1024 * 1024 * 1024 {
			// 1 GB limit
			return Err(CommonError::InvalidArgument {
				ArgumentName:"Content".to_string(),
				Reason:"Content exceeds maximum size limit of 1GB".to_string(),
			});
		}

		let PathExists = fs::try_exists(Path).await.unwrap_or(false);

		if PathExists && !Overwrite {
			return Err(CommonError::FileSystemFileExists(Path.clone()));
		}

		if !PathExists && !Create {
			return Err(CommonError::FileSystemNotFound(Path.clone()));
		}

		// Create parent directories if they don't exist
		if let Some(ParentDirectory) = Path.parent() {
			if !fs::try_exists(ParentDirectory).await.unwrap_or(false) {
				fs::create_dir_all(ParentDirectory).await.map_err(|Error| {
					CommonError::FromStandardIOError(Error, ParentDirectory.to_path_buf(), "WriteFile.CreateParent")
				})?;
			}
		}

		fs::write(Path, &Content)
			.await
			.map_err(|Error| CommonError::FromStandardIOError(Error, Path.clone(), "WriteFile"))?;

		// Implement atomic write pattern to prevent partial writes and data corruption
		// on crashes or interrupts. The current implementation writes directly to the
		// target file, which can leave corrupted files if the operation is interrupted.
		// A robust implementation: 1) writes content to a temporary file in the same
		// directory (ensuring same filesystem for atomic rename), 2) flushes and syncs
		// the temporary file to disk (fsync), 3) atomically renames the temporary file
		// to the target path using fs::rename (POSIX rename is atomic within a
		// filesystem), 4) deletes old file if replacing, or handles temp cleanup on
		// failure. This pattern ensures the target file is either fully written or
		// unchanged.
		Ok(())
	}

	/// Creates a directory after verifying access rights.
	/// When Recursive is true, creates all parent directories.
	/// Fails if directory already exists.
	async fn CreateDirectory(&self, Path:&PathBuf, Recursive:bool) -> Result<(), CommonError> {
		Utility::IsPathAllowedForAccess(&self.ApplicationState, Path)?;

		// Validate that parent path doesn't point to a file
		if let Some(ParentPath) = Path.parent().filter(|p| !p.as_os_str().is_empty()) {
			if fs::try_exists(ParentPath).await.unwrap_or(false) {
				let ParentMetadata = fs::metadata(ParentPath).await.map_err(|Error| {
					CommonError::FromStandardIOError(Error, ParentPath.to_path_buf(), "CreateDirectory.ParentStat")
				})?;

				if ParentMetadata.is_file() {
					return Err(CommonError::InvalidArgument {
						ArgumentName:"Path".to_string(),
						Reason:format!("Cannot create directory: parent path is a file: {}", ParentPath.display()),
					});
				}
			}
		}

		let Operation = if Recursive {
			fs::create_dir_all(Path).await
		} else {
			fs::create_dir(Path).await
		};

		Operation.map_err(|Error| CommonError::FromStandardIOError(Error, Path.clone(), "CreateDirectory"))
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

				Operation.map_err(|Error| CommonError::FromStandardIOError(Error, Path.clone(), "Delete"))
			},

			// Idempotent success
			Err(Error) if Error.kind() == std::io::ErrorKind::NotFound => Ok(()),

			Err(Error) => Err(CommonError::FromStandardIOError(Error, Path.clone(), "Delete.Stat")),
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
			.map_err(|Error| CommonError::FromStandardIOError(Error, Source.clone(), "Rename"))
	}

	/// Copies a file after verifying access rights.
	/// Currently does not support recursive directory copying.
	async fn Copy(&self, Source:&PathBuf, Target:&PathBuf, Overwrite:bool) -> Result<(), CommonError> {
		Utility::IsPathAllowedForAccess(&self.ApplicationState, Source)?;

		Utility::IsPathAllowedForAccess(&self.ApplicationState, Target)?;

		// Validate that source exists
		if !fs::try_exists(Source).await.unwrap_or(false) {
			return Err(CommonError::FileSystemNotFound(Source.clone()));
		}

		let SourceMetadata = self.StatFile(Source).await?;

		if (SourceMetadata.FileType & FileTypeDTO::Directory as u8) != 0 {
			return Err(CommonError::NotImplemented { FeatureName:"Recursive directory copy".to_string() });
		}

		// Prevent copying file to itself (which would truncate it)
		if fs::canonicalize(Source).await.ok().as_ref() == fs::canonicalize(Target).await.ok().as_ref() {
			return Err(CommonError::InvalidArgument {
				ArgumentName:"Target".to_string(),
				Reason:"Cannot copy file to itself".to_string(),
			});
		}

		if !Overwrite && fs::try_exists(Target).await.unwrap_or(false) {
			return Err(CommonError::FileSystemFileExists(Target.clone()));
		}

		// Create target parent directory if needed
		if let Some(TargetParent) = Target.parent() {
			if !fs::try_exists(TargetParent).await.unwrap_or(false) {
				fs::create_dir_all(TargetParent).await.map_err(|Error| {
					CommonError::FromStandardIOError(Error, TargetParent.to_path_buf(), "Copy.CreateTargetParent")
				})?;
			}
		}

		fs::copy(Source, Target)
			.await
			.map(|_| ())
			.map_err(|Error| CommonError::FromStandardIOError(Error, Source.clone(), "Copy"))
	}

	/// Creates a new, empty file after verifying access rights.
	/// Fails if the file already exists (use WriteFile with Overwrite to
	/// replace).
	async fn CreateFile(&self, Path:&PathBuf) -> Result<(), CommonError> {
		// Use WriteFile with an empty Vec, ensuring creation without overwrite.
		// This ensures proper parent directory creation and path validation.
		self.WriteFile(Path, vec![], true, false).await
	}
}
