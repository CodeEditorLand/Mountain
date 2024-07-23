#[tauri::command]
pub async fn Get(
	Path: String,
	Work: tauri::State<'_, std::sync::Arc<Echo::Fn::Job::Work>>,
) -> Result<(), String> {
	crate::Fn::Get::Fn(Path, Work).await
}

#[tauri::command]
pub async fn Put(
	Path: String,
	Content: String,
	Work: tauri::State<'_, std::sync::Arc<Echo::Fn::Job::Work>>,
) -> Result<(), String> {
	crate::Fn::Put::Fn(Path, Content, Work).await
}
