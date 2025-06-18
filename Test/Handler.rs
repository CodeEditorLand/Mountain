// @module handler_tests
// @description This file demonstrates how to write unit tests for Mountain's
// handler logic using the `mockall` crate for dependency mocking.

// This code would live in a file under the `tests/` directory in the Mountain crate.
// It requires `mockall` to be added as a `[dev-dependency]`.

#[cfg(test)]
mod tests {
	use std::{path::PathBuf, sync::Arc};

	use Common::{
		error::CommonError,
		fs::{
			FileSystemReader,
			DTO::{FileSystemStatDTO, FileTypeDTO},
		},
	};
	use async_trait::async_trait;
	use mockall::automock;

	// --- Mocking the Trait ---
	// We create a dummy struct and impl the trait we want to mock.
	// The `#[automock]` attribute will generate `MockTestFileSystemReader` for us.
	#[automock]
	#[async_trait]
	pub trait TestFileSystemReader {
		async fn ReadFile(&self, Path:&PathBuf) -> Result<Vec<u8>, CommonError>;
	}

	// --- The Function/Handler Under Test ---
	// This is a simplified handler that depends on any type that implements
	// `FileSystemReader`. Thanks to generics, we can pass in a real FileSystemReader or our mock
	// one.
	async fn LogicThatReadsAFile(
		Reader:Arc<dyn TestFileSystemReader + Send + Sync>,
		Path:PathBuf,
	) -> Result<String, CommonError> {
		let Bytes = Reader.ReadFile(&Path).await?;
		Ok(String::from_utf8(Bytes).unwrap_or_default())
	}

	// --- The Test Case ---
	#[tokio::test]
	async fn Test_LogicThatReadsAFile_ReturnsCorrectString_OnSuccess() {
		// 1. Arrange: Create the mock object.
		let mut MockReader = MockMockTestFileSystemReader::new();

		// 2. Arrange: Set up an expectation.
		// We expect the `ReadFile` method to be called exactly once.
		// When it is, we tell it to return `Ok("hello".to_vec())`.
		MockReader
			.expect_ReadFile()
			.times(1)
			.returning(|_path| Ok("hello".as_bytes().to_vec()));

		// 3. Act: Call our handler function with the mock dependency.
		let result = LogicThatReadsAFile(Arc::new(MockReader), PathBuf::from("/fake/path.txt")).await;

		// 4. Assert: Check that the result is what we expect.
		assert!(result.is_ok());
		assert_eq!(result.unwrap(), "hello");
	}

	#[tokio::test]
	async fn Test_LogicThatReadsAFile_ReturnsError_OnFailure() {
		// 1. Arrange: Create the mock object.
		let mut MockReader = MockMockTestFileSystemReader::new();

		// 2. Arrange: Set up an expectation for failure.
		// We tell the mock to return an FileSystemNotFound error when called.
		MockReader
			.expect_ReadFile()
			.times(1)
			.returning(|path| Err(CommonError::FileSystemNotFound(path.clone())));

		// 3. Act: Call our handler function.
		let path = PathBuf::from("/not/found.txt");
		let result = LogicThatReadsAFile(Arc::new(MockReader), path.clone()).await;

		// 4. Assert: Check that the result is the correct error.
		assert!(result.is_err());
		match result.unwrap_err() {
			CommonError::FileSystemNotFound(p) => assert_eq!(p, path),
			_ => panic!("Expected FileSystemNotFound error"),
		}
	}
}
