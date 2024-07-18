/// Enqueues a write action to the work queue.
///
/// This function is a Tauri command that takes a file path and content, and assigns a `Write` action
/// to the work queue. The action will be processed asynchronously by a worker.
///
/// # Arguments
///
/// * `Path` - A `String` representing the file path to write to.
/// * `Content` - A `String` representing the content to be written to the file.
/// * `Work` - A Tauri state containing an `Arc` reference to a `Work` instance, which holds the queue of actions to be processed.
///
/// # Returns
///
/// A `Result` indicating the success or failure of the operation. Returns `Ok(())` if the action was successfully assigned to the queue.
///
/// # Errors
///
/// This function will return an `Err` if there is an issue assigning the action to the queue.
///
#[tauri::command]
pub async fn Fn(
	Path: String,
	Content: String,
	Work: tauri::State<'_, std::sync::Arc<Echo::Fn::Job::Work>>,
) -> Result<(), String> {
	Work.Assign(Echo::Fn::Job::Action::Write { Path, Content }).await;

	Ok(())
}
