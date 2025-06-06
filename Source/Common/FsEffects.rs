// File: Common/FsEffect.rs
// Defines traits and effects for interacting with the filesystem.
// This provides a standardized, asynchronous way to read from and write to the
// filesystem.

#![allow(non_snake_case, non_camel_case_types)]

use std::{path::PathBuf, sync::Arc};

use async_trait::async_trait;

use crate::FileSystemDto::{FileSystemStat, FileType}; // Assuming DTOs are here
use crate::{
	Effect::ActionEffect,
	Environment::{Environment, Requires},
	Errors::CommonError,
	Runtime::AppRuntimeTrait,
};

/// A trait for environments that can read from the filesystem.
#[async_trait]
pub trait FsReader: Environment + Send + Sync {
	/// Reads the entire contents of a file into a bytes vector.
	async fn ReadFile(&self, Path:&PathBuf) -> Result<Vec<u8>, CommonError>;
	/// Retrieves metadata for a file or directory.
	async fn StatFile(&self, Path:&PathBuf) -> Result<FileSystemStat, CommonError>;
	/// Reads the contents of a directory, returning a list of entries with
	/// their types.
	async fn ReadDirectory(&self, Path:&PathBuf) -> Result<Vec<(String, FileType)>, CommonError>;
}

/// A trait for environments that can write to the filesystem.
#[async_trait]
pub trait FsWriter: Environment + Send + Sync {
	/// Writes a slice of bytes to a file.
	async fn WriteFile(&self, Path:&PathBuf, Content:Vec<u8>, Create:bool, Overwrite:bool) -> Result<(), CommonError>;
	/// Creates a new directory.
	async fn CreateDirectory(&self, Path:&PathBuf, Recursive:bool) -> Result<(), CommonError>;
	/// Deletes a file or directory.
	async fn Delete(&self, Path:&PathBuf, Recursive:bool, UseTrash:bool) -> Result<(), CommonError>;
	/// Renames or moves a file or directory.
	async fn Rename(&self, Source:&PathBuf, Target:&PathBuf, Overwrite:bool) -> Result<(), CommonError>;
	/// Copies a file. Directory copy may not be supported by all
	/// implementations.
	async fn Copy(&self, Source:&PathBuf, Target:&PathBuf, Overwrite:bool) -> Result<(), CommonError>;
	/// Creates a new, empty file.
	async fn CreateFile(&self, Path:&PathBuf) -> Result<(), CommonError>;
}

// --- FsReader Effect ---

pub fn ReadFile<RuntimeAccessType>(Path:PathBuf) -> ActionEffect<Arc<RuntimeAccessType>, CommonError, Vec<u8>>
where
	RuntimeAccessType: AppRuntimeTrait<RuntimeAccessType::EnvironmentType> + Send + Sync + 'static,
	RuntimeAccessType::EnvironmentType: Requires<Arc<dyn FsReader>>, {
	ActionEffect::New(Arc::new(move |Accessor| {
		let PathClone = Path.clone();
		Box::pin(async move {
			let Environment = Accessor.GetEnvironment();
			let Reader:Arc<dyn FsReader> = Environment.require();
			Reader.ReadFile(&PathClone).await
		})
	}))
}

pub fn StatFile<RuntimeAccessType>(Path:PathBuf) -> ActionEffect<Arc<RuntimeAccessType>, CommonError, FileSystemStat>
where
	RuntimeAccessType: AppRuntimeTrait<RuntimeAccessType::EnvironmentType> + Send + Sync + 'static,
	RuntimeAccessType::EnvironmentType: Requires<Arc<dyn FsReader>>, {
	ActionEffect::New(Arc::new(move |Accessor| {
		let PathClone = Path.clone();
		Box::pin(async move {
			let Environment = Accessor.GetEnvironment();
			let Reader:Arc<dyn FsReader> = Environment.require();
			Reader.StatFile(&PathClone).await
		})
	}))
}

pub fn ReadDirectory<RuntimeAccessType>(
	Path:PathBuf,
) -> ActionEffect<Arc<RuntimeAccessType>, CommonError, Vec<(String, FileType)>>
where
	RuntimeAccessType: AppRuntimeTrait<RuntimeAccessType::EnvironmentType> + Send + Sync + 'static,
	RuntimeAccessType::EnvironmentType: Requires<Arc<dyn FsReader>>, {
	ActionEffect::New(Arc::new(move |Accessor| {
		let PathClone = Path.clone();
		Box::pin(async move {
			let Environment = Accessor.GetEnvironment();
			let Reader:Arc<dyn FsReader> = Environment.require();
			Reader.ReadDirectory(&PathClone).await
		})
	}))
}

// --- FsWriter Effect ---

pub fn WriteFileBytes<RuntimeAccessType>(
	Path:PathBuf,
	Content:Vec<u8>,
	Create:bool,
	Overwrite:bool,
) -> ActionEffect<Arc<RuntimeAccessType>, CommonError, ()>
where
	RuntimeAccessType: AppRuntimeTrait<RuntimeAccessType::EnvironmentType> + Send + Sync + 'static,
	RuntimeAccessType::EnvironmentType: Requires<Arc<dyn FsWriter>>, {
	ActionEffect::New(Arc::new(move |Accessor| {
		let PathClone = Path.clone();
		let ContentClone = Content.clone();
		Box::pin(async move {
			let Environment = Accessor.GetEnvironment();
			let Writer:Arc<dyn FsWriter> = Environment.require();
			Writer.WriteFile(&PathClone, ContentClone, Create, Overwrite).await
		})
	}))
}

pub fn WriteFileString<RuntimeAccessType>(
	Path:PathBuf,
	Content:String,
	Create:bool,
	Overwrite:bool,
) -> ActionEffect<Arc<RuntimeAccessType>, CommonError, ()>
where
	RuntimeAccessType: AppRuntimeTrait<RuntimeAccessType::EnvironmentType> + Send + Sync + 'static,
	RuntimeAccessType::EnvironmentType: Requires<Arc<dyn FsWriter>>, {
	WriteFileBytes(Path, Content.into_bytes(), Create, Overwrite)
}

pub fn CreateDirectory<RuntimeAccessType>(
	Path:PathBuf,
	Recursive:bool,
) -> ActionEffect<Arc<RuntimeAccessType>, CommonError, ()>
where
	RuntimeAccessType: AppRuntimeTrait<RuntimeAccessType::EnvironmentType> + Send + Sync + 'static,
	RuntimeAccessType::EnvironmentType: Requires<Arc<dyn FsWriter>>, {
	ActionEffect::New(Arc::new(move |Accessor| {
		let PathClone = Path.clone();
		Box::pin(async move {
			let Environment = Accessor.GetEnvironment();
			let Writer:Arc<dyn FsWriter> = Environment.require();
			Writer.CreateDirectory(&PathClone, Recursive).await
		})
	}))
}

pub fn CreateFile<RuntimeAccessType>(Path:PathBuf) -> ActionEffect<Arc<RuntimeAccessType>, CommonError, ()>
where
	RuntimeAccessType: AppRuntimeTrait<RuntimeAccessType::EnvironmentType> + Send + Sync + 'static,
	RuntimeAccessType::EnvironmentType: Requires<Arc<dyn FsWriter>>, {
	ActionEffect::New(Arc::new(move |Accessor| {
		let PathClone = Path.clone();
		Box::pin(async move {
			let Environment = Accessor.GetEnvironment();
			let Writer:Arc<dyn FsWriter> = Environment.require();
			Writer.CreateFile(&PathClone).await
		})
	}))
}

pub fn Delete<RuntimeAccessType>(
	Path:PathBuf,
	Recursive:bool,
	UseTrash:bool,
) -> ActionEffect<Arc<RuntimeAccessType>, CommonError, ()>
where
	RuntimeAccessType: AppRuntimeTrait<RuntimeAccessType::EnvironmentType> + Send + Sync + 'static,
	RuntimeAccessType::EnvironmentType: Requires<Arc<dyn FsWriter>>, {
	ActionEffect::New(Arc::new(move |Accessor| {
		let PathClone = Path.clone();
		Box::pin(async move {
			let Environment = Accessor.GetEnvironment();
			let Writer:Arc<dyn FsWriter> = Environment.require();
			Writer.Delete(&PathClone, Recursive, UseTrash).await
		})
	}))
}

pub fn Rename<RuntimeAccessType>(
	Source:PathBuf,
	Target:PathBuf,
	Overwrite:bool,
) -> ActionEffect<Arc<RuntimeAccessType>, CommonError, ()>
where
	RuntimeAccessType: AppRuntimeTrait<RuntimeAccessType::EnvironmentType> + Send + Sync + 'static,
	RuntimeAccessType::EnvironmentType: Requires<Arc<dyn FsWriter>>, {
	ActionEffect::New(Arc::new(move |Accessor| {
		let SourceClone = Source.clone();
		let TargetClone = Target.clone();
		Box::pin(async move {
			let Environment = Accessor.GetEnvironment();
			let Writer:Arc<dyn FsWriter> = Environment.require();
			Writer.Rename(&SourceClone, &TargetClone, Overwrite).await
		})
	}))
}

pub fn Copy<RuntimeAccessType>(
	Source:PathBuf,
	Target:PathBuf,
	Overwrite:bool,
) -> ActionEffect<Arc<RuntimeAccessType>, CommonError, ()>
where
	RuntimeAccessType: AppRuntimeTrait<RuntimeAccessType::EnvironmentType> + Send + Sync + 'static,
	RuntimeAccessType::EnvironmentType: Requires<Arc<dyn FsWriter>>, {
	ActionEffect::New(Arc::new(move |Accessor| {
		let SourceClone = Source.clone();
		let TargetClone = Target.clone();
		Box::pin(async move {
			let Environment = Accessor.GetEnvironment();
			let Writer:Arc<dyn FsWriter> = Environment.require();
			Writer.Copy(&SourceClone, &TargetClone, Overwrite).await
		})
	}))
}
